# Vism既知実装採択マップ

状態: **決定／調査結果固定。依存追加・runtime実装・公開契約の許可ではない**（2026-08-02）。

本書は[Vism入口・並列解禁の根本マップ](reviews/2026-08-02-vism-entrance-parallelization-root-map.md)の各入口を、
[既知実装採択・置換開発モデル](known-implementation-adoption-model.md)の
`既知解 → 採用方式 → private境界 → probe → cutover → retirement`へ写す。
Vismの表現意味、Host owner、Document single writer、VRAM常駐、色変換一元化はMotoliiが所有する。
compiler、geometry、layout、solver、sandbox、配布保護等の一般機構は、ここで裁定した既知解を継承し、
各Vism実装が再発明しない。

## 1. この地図が決めること

- 同じ機構classの後続粒は、本書のrouteとcandidate pinを継承する。
- `REUSE`以外の候補は、明記したprivate adapterまたは外部oracleから開始する。probe合格前に製品依存へ入れない。
- probe失敗は既存の解決段へ戻る。ここでいう`REMAP`は別の既存target／routeへ型付きで写し直す**操作**であり、
  新しい状態語ではない。成果を保つ最小sliceへの`REDUCE`と同様、独自frameworkを作る許可ではない。
- `PATTERN`は設計先例、`EXTERNAL`はprocess／tool／検証oracleであり、source、型、state ownerを持ち込まない。
- cutoverは同一Motolii fixtureで新旧routeを比較し、ownerを一回だけ切り替える。旧routeは`FROZEN`後に`RETIRE`する。
- 本書は`VSM-A9`前の並列実装、Vism package、loader、manifest、公開Rust／TypeScript API、Document／serdeを解禁しない。

## 2. 固定candidate registry

| ID | 固定した既知解 | 参照する具体面 | license／採否の上限 |
|---|---|---|---|
| `K-WGPU` | workspace [`wgpu 29`](https://wgpu.rs/doc/wgpu/)＋transitive Naga | shader module、bind group、pipeline、validation | MIT OR Apache-2.0。`REUSE`。GPU ownerはHostのまま |
| `K-VECTOR` | workspace [`vello 0.9.0`](https://github.com/linebender/vello)＋transitive [Kurbo](https://github.com/linebender/kurbo) | Vello scene/glyph描画、Kurbo path/stroke/offset | MIT OR Apache-2.0。`REUSE`。Vism公開型へ漏らさない |
| `K-TEXT` | workspace [`fontique 0.10`](https://github.com/linebender/fontique)＋[`harfrust 0.7`](https://github.com/harfbuzz/harfrust) | font discovery、shaping、glyph ID／position | MIT。`REUSE`候補を継承するがText routeは未成立。[Parley `0.11.0`](https://docs.rs/parley/0.11.0/parley/)は縦書き負例のため全体採用しない |
| `K-LYON` | [`lyon 1.0.19`](https://docs.rs/lyon/1.0.19/lyon/) | tessellation API | MIT OR Apache-2.0。`WRAP`候補、Path意味の正本にはしない |
| `K-ARROW` | [`arrow 59.1.0`](https://docs.rs/arrow/59.1.0/arrow/) | columnar array／IPC | Apache-2.0。高量Dataの閾値反証候補であり、現時点では`REJECT / NO-PROBE` |
| `K-GLTF` | [glTF 2.0](https://www.khronos.org/gltf/)、[`gltf 1.4.1`](https://docs.rs/gltf/1.4.1/gltf/)、[`gltf-validator 2.0.0-dev.3.10`](https://www.npmjs.com/package/gltf-validator/v/2.0.0-dev.3.10) | import、buffer／accessor、validation report | Apache-2.0/MIT。private importer＋固定版`EXTERNAL` oracle |
| `K-PHYSICS` | [`parry3d 0.29.0`](https://docs.rs/parry3d/0.29.0/parry3d/)、[`rapier3d 0.34.0`](https://docs.rs/rapier3d/0.34.0/rapier3d/) | collision query、rigid-body step、`enhanced-determinism`比較 | Apache-2.0。domain solverだけの`WRAP`候補 |
| `K-TS` | [TypeScript `7.0.2` compiler／Language Service](https://github.com/microsoft/TypeScript/wiki/Using-the-Compiler-API) | AST、typecheck、diagnostic、incremental authoring | Apache-2.0。隔離authoring toolchainへ`ADOPT`。runtimeではない |
| `K-JS` | QuickJS系 [`rquickjs 0.12.2`](https://docs.rs/rquickjs/0.12.2/rquickjs/) | runtime、interrupt、memory limit、module loader | MIT。`VSM-C2` private probe候補。Boa／`deno_core`を黙って代替しない |
| `K-WASM` | [`wasmtime 47.0.3`](https://docs.rs/wasmtime/47.0.3/wasmtime/) component model | component boundary、fuel／epoch interruption、resource limit | Apache-2.0 WITH LLVM-exception。private probe候補。Rust 1.94要求をtoolchain gateにする |
| `K-GRAPH` | [MaterialX `1.39.4`](https://github.com/AcademySoftwareFoundation/MaterialX/blob/v1.39.4/documents/Specification/MaterialX.Specification.md)、[Blender `4.5 LTS` Geometry Nodes](https://docs.blender.org/manual/en/4.5/modeling/geometry_nodes/) | typed graph、field evaluation、simulation zone、group interface | `PATTERN`だけ。Motolii schema／node graphを逆算しない |
| `K-INTEROP` | [JSON-RPC 2.0](https://www.jsonrpc.org/specification)、[LSP 3.18](https://github.com/microsoft/language-server-protocol/blob/gh-pages/_specifications/lsp/3.18/specification.md) | request identity、capability negotiation、cancel | `PATTERN`だけ。Host D2、permission、Document ownerは移さない |
| `K-SUPPLY` | [OCI Image Spec `1.1.1` descriptor](https://github.com/opencontainers/image-spec/blob/v1.1.1/descriptor.md)、[TUF spec `1.0.34`](https://github.com/theupdateframework/specification/blob/v1.0.34/tuf-spec.md) | digest＋size＋media type、rollback/freeze protection、offline verification | `PATTERN`だけ。container filesystem、registry、中央service依存を採らない |
| `K-INDUSTRY` | [OpenFX `1.5.1` (`ab779510`)](https://github.com/AcademySoftwareFoundation/openfx/tree/OFX_Release_1.5.1/Documentation)、[ISF specification `2.0`](https://docs.isf.video/ref_changes.html) | image-effect lifecycle、JSON/pass宣言 | `PATTERN`だけ。OFX ABI、ISF runtimeを製品へ持ち込まない |

crate版は2026-08-02のregistry metadata、workspace版は同日の`Cargo.toml`を固定根拠とする。
外部仕様は各リンク先のversionをprobe orderでhash固定する。hashを固定できない候補は実装担当へ渡さない。

調査順はrepo内既存dependency／実装、`decision-index.md`、`references.md`、上表の一次資料とした。
repo照合で`wgpu / Vello / Fontique / HarfRust`は導入済み、Lyon、Arrow、glTF、Parry、Rapier、rquickjs、Wasmtimeは
未導入と確認した。一次資料が証明するのは各API、version、license、制約の候補事実までであり、Motoliiでの性能、
thread safety、state owner、3 OS可搬性、security、製品適合は証明しない。これらは§6のprobe order必須欄で閉じる。

## 3. 描画・評価入口

| 利用者成果／入口 | 採用方式と接続先 | private境界 | 最小probeと合否oracle | cutover／retirement | 再調査条件 |
|---|---|---|---|---|---|
| 単一pass Filter | `REUSE K-WGPU`。既存`FilterPlugin`、`RenderCtx`、`PipelineCache`へ接続 | shader／pipelineはHost所有。Vismはtyped parameterとtexture in/outだけ | 現行purity＋pixel fixture、Draft/Final同一関数、loop内resource生成0、CPU readback 0 | 新routeなし。現行routeを`KEEP` | binding shape、texture count、budgetが現行契約を越えた時だけA8G0へ |
| multipass Filter／Texture | `PATTERN K-INDUSTRY`。ISFのpass宣言とOpenFX lifecycleをA8G0の反例に使い、実装は`REUSE K-WGPU` | pass graph、transient texture、lifetime、cache key、budgetはHost private。plugin内pool禁止 | Glow＋Blurの二fixture、linear/HDR intermediate、mask、低予算typed reject、resource再利用、色変換0 | 現在の退役対象は0。使用中の公開`get_or_create_fullscreen_uniform16`は`KEEP`し、A8G2/A8G3のRadial Repeater parityと独立公開境界gateなしに変更・退役しない | 二fixtureで共通化不能、wgpu制約、3 OS差、color oracle不一致なら`REMAP / REDUCE` |
| Composite | `REUSE K-WGPU`＋`PATTERN K-INDUSTRY`。既存`CompositePlugin`へ接続 | 入力fan-inとpass schedulingだけHost private。Document／scene graphを追加しない | 2入力、alpha、mask、順序、欠落texture、budget拒否をCPU独立oracleと比較 | 現在の退役対象は0。将来重複routeが実在した場合だけ同fixture後に処分 | Filterと責任差をtyped fixtureで表せない時はA8G0へ戻す |
| LayerSource／Generator | `REUSE K-WGPU`。既存prepared lowering、Radial Repeater、Host cacheへ接続 | 0-input recipe→textureだけ。作者crateからHost private crateを見せない | A3 purity／golden、外部crate private依存拒否、同一`t`同一出力 | 現行一般loweringを`KEEP`。新resource shape用の複製helperは作らない | 現行公開façadeで書けない時は独立Host capability仕様へ |
| Parameter provider | 既存`ParamDriverPlugin`と`DataTrack → ParamSource::Data`を`REUSE` | providerは決定的trackを返す。consumer port、event、Document writerを追加しない | 固定BPM／RationalTime、missing、同一入力同一値、既存parameter評価との同値 | 現行routeを`KEEP`。provider-onlyのため退役対象0 | consumer portが必要ならB2へ。provider identityを推測しない |
| Data provider→consumer | 現行`DataTrack → ParamSource::Data`を`REUSE`し、`K-ARROW`は`REJECT / NO-PROBE`。B2で高量の実在成果と測定閾値が固定された時だけ再調査 | 将来probeでもArrow型はadapter crate内。Document、plugin API、serdeへ出さない | B2の同じRationalTime列で値／seek同値、allocation、missing、unknownを測る。閾値未達はno-op終了 | 現在のownerは既存DataTrackだけでrouteは`KEEP`。二重内部表現を常設せず、採択時は一ownerへcutover | B2が高量の実在成果と測定済み閾値を固定した時だけ`K-ARROW`を再入場 |

## 4. 意味値・表現入口

| 利用者成果／入口 | 採用方式と接続先 | private境界 | 最小probeと合否oracle | cutover／retirement | 再調査条件 |
|---|---|---|---|---|---|
| Path2D | native `pathgeom::apply`だけを`REUSE` | SDK-S0 consumer-neutral fixtureはtest-only。Kurbo／Vello／Lyon／GPU tessellationをPath意味へ入れない | SDK-S0 positive 4／negative 7、closed contour、offset、finite、native oracle同値 | native意味ownerを`KEEP`。現在のcutover／退役対象0 | unsupported operation、robustness、oracle不一致が出た時 |
| Shape2D | Path2D、現行`VectorRecipe`、`K-VECTOR`を`REUSE`。`K-LYON`は描画tessellation gapだけ`WRAP`候補 | group/style/operator loweringとtessellation adapterはprivate。外部shape schema／Lyon型をDocument／plugin contractにしない | fill/stroke/group/transform/operator順序、empty、self-intersection、canonical spaceを既存goldenへ。Lyonは同じShape画素oracleで比較 | 現行`VectorRecipe` ownerを`KEEP`。Lyon採用時も実在する重複tessellatorだけ同oracle後に退役 | 新しい保存意味、unsupported operator、tessellation robustness、golden不一致ならD2仕様へ戻す |
| Text | 採択済み候補`K-TEXT`＋`K-VECTOR`を後続owner決定へ継承するが、接続targetは`ABSENT`。Parley 0.11は水平layoutの`PATTERN`に限定し、whole-stackは`REJECT` | run／cluster／glyphとfont admissionはowner決定までprivate。Parley型を公開しない | owner決定後だけ、横書き、縦書き、ruby、bidi、fallback、variable font、missing fontをHarfRust glyph列とVello描画で別判定 | Text routeは未成立で、現在のcutover／退役対象は0 | text shaping／font admission owner、cluster identity、P0Iとの関係が閉じた時 |
| Instance | Blender Geometry Nodesを`PATTERN`。実装依存はまだ採らない | stable identity、prototype、nesting、channelはM5-P0I docs fixture内。renderer型をidentityにしない | rename/update/duplicate/missing、ordinal再配置、nested instance、同一prototypeを表だけで反証 | `InstanceIndex`を`FROZEN`なstable identityへ昇格せず、P0I採択後に互換adapterの退役条件を決める | P0Iのdomain identity決定、M5 fixtureの反例、provider lifetime変更時 |
| Spatial | Blender Geometry Nodesのspace-tagged operationを`PATTERN`。接続targetはM5 camera／geometry／rendererで、現状`ABSENT` | canonical XYZ、camera、depth、bounds、surfaceはHost owner。importer／renderer型を意味SDKへ出さない | M5 owner決定後だけspace、unit、axis、transform、bounds、camera、depthのsemantic fixture | runtime未成立のためcutover／退役対象0 | M5 camera／geometry／renderer ownerと中間形式が閉じた時 |
| 3D import adapter | `WRAP K-GLTF`＋Khronos Validatorを`EXTERNAL`。Spatial意味やrenderer採択とは別grain | imported asset→採択済みcanonical Motolii geometryへの変換だけ。gltf型、scene graph、camera、materialを公開面へ漏らさない | M5中間形式決定後だけKhronos sample＋invalid corpus、NaN、bounds、axis、units、missing resource、unknown extension、Validator report同値 | 現在の退役対象は0。採択probeの一時decoderは`DELETE-LATER`を発注時に固定する | unsupported extension、material/color mismatch、streaming、M5中間形式変更時 |
| Field | Blender Fieldsを`PATTERN`。接続targetとなるField／Collider representationは現状`ABSENT` | scalar／vector field、mask、SDF正規化、budgetのownerはHost。collision engine型をField意味にしない | representation決定後だけpoint sample、space、finite、transform、composition、budgetを解析oracleと比較 | runtime未成立のためcutover／退役対象0 | Field／Collider representationとcanonical-space ownerが閉じた時 |
| collision query | `parry3d 0.29.0`をcollision queryだけ`WRAP`候補。Field意味やSimulation solverとは別grain | collider adapter、acceleration structureはHost private。Parry型をDocument／public APIへ出さない | collider representation決定後だけdistance、inside/outside、degenerate、NaN、determinismを解析oracleと比較 | 現在の退役対象は0。手製solverを先に作らず、probe artifactは`DELETE-LATER` | 2D/3D差、determinism、representation、platform条件が変わった時 |
| Simulation | Blender Simulation Zoneを`PATTERN`、`rapier3d 0.34.0`をrigid/collision solverだけ`WRAP`候補 | HostがBake／StateTrack／checkpoint／invalidateを所有。Rapier worldをplugin state、Document、public APIにしない | SIM-1/VSM-A6後だけfixed seed/time step、checkpoint restore、invalidate、Cancel、budget、missing solver、3 OS、`enhanced-determinism`比較 | owner成立前のsolverは作らない。採択時はsolver seatだけ一回切替し、probe artifactを`RETIRE`。Host state routeは`KEEP` | determinism不成立、solver domain外、MSRV／platform、StateTrack意味衝突時 |

## 5. 作者・接続・配布入口

| 利用者成果／入口 | 採用方式と接続先 | private境界 | 最小probeと合否oracle | cutover／retirement | 再調査条件 |
|---|---|---|---|---|---|
| TypeScript authoring | `ADOPT K-TS`。LANG-TS-F0のheadless compiler／Language Serviceへ接続 | Node toolchain、generated declaration、diagnostic mapperはauthoring harness内。製品runtime／Cargo／Documentへ入れない | SDK-S0同値、closed allowlist、MTS-N1〜N5/N7/N9、典型error 8/10、10分fork | F0合格後に手製parser／string scan案を`REJECT`。F1まではlive routeなし | TS major、diagnostic drift、offline pin、license／toolchain、fixture不一致時 |
| JavaScript live runtime | `WRAP K-JS`をまずVSM-C2 sandboxでprivate probeし、その合格候補だけを後続LANG-TS-F1 feedback probeへ渡す。Boaはexperimental反例、`deno_core`は重量比較だけ | engine handle、module loader、interrupt、memory、globals、last-goodはruntime adapter内。engine objectを公開しない | C2はdeadline、memory、forbidden FS/network/process、deterministic globals、crash隔離、3 OS。F1だけがlast-goodとedit-to-preview p50を判定 | C2と後続F1の両方に合格したengineへ一回切替しprobe harnessを退役。不合格ならlive JSを`REDUCE`しF0 authoringだけ残す | CVE、maintenance、ECMAScript gap、startup/binary budget、sandbox bypass時 |
| admitted WASM module | `WRAP K-WASM`。JS runtimeとは別seat | component instance、resource limiter、fuel／epoch、capability importsはHost private。component interfaceをVism manifestに先行固定しない | C2でtimeout、memory、trap、forbidden capability、determinism、unload、他Vism継続、Wasmtime 47のtoolchain適合を測る | C2結果を含むC0〜C3後のC4採択までruntime adapterを昇格しない。旧native-only仮定は`FROZEN`、probe artifactsは`RETIRE` | Rust/toolchain不適合、component model変更、binary size、platform、security advisory時 |
| Generator／Materialize | 既存`DocumentWriter::apply_macro`を`REUSE`し、未成立のU9a／B2I全体preflightをその手前へ接続する。新workflow engineを作らない | proposal、全体preflight、開始snapshotはHost private。Vismはtyped proposalまで | B2方式決定後、途中失敗／type mismatch／cycle／Cancel／staleでDocument・history変更0、一回の既存`apply_macro`／一Undo | 現在の退役対象は0。全体preflight無しの逐次materialize案を`REJECT`し、D2 single writerと既存`apply_macro`は`KEEP` | B2方式、U9a／B2I preflight、single-writer oracleが未閉鎖なら実装しない |
| External Bridge | `PATTERN K-INTEROP`。app外接続はJSON-RPC/LSP形に限定 | transport/session/capability adapter内。外部toolはtyped proposalまで、Document変更はHost D2だけ | request identity、cancel、stale、permission、partial failure、unknown capability | 現在の退役対象は0。実在app固有routeを置換する粒で初めてretirementを固定 | app protocol差、identity衝突、permission owner未閉鎖時 |
| Automation | `PATTERN K-INTEROP`のrequest/cancelを後続候補へ継承するが、接続targetは`ABSENT` | immutable snapshot、explicit target、proposal、consentは将来Authoring Tool seat＋Host。Bridge transportやpackageと共有しない | owner決定後だけstale、permission、Cancel、Document部分一macro、Bake／Export job分離、open時no-installを反証 | runtime未成立のためcutover／退役対象0 | proposal owner、permission、atomic batch、job分離が閉じた時 |
| Kit／Preset／typed graph | `PATTERN K-GRAPH`。Node Groups／MaterialX NodeDefのinterface、provider選択、defaultをB0〜B2の反例に使う | graph evaluator、node schema、UIは持ち込まない。Kit identityとmaterialized Documentを分離 | rename/update/missing/cycle/type mismatch/default/fork、materialize前全体preflight、一macro／一Undo | 独立Plugin Setは既に`RETIRED`のまま。現在の追加退役対象は0、採択前は形式0 | B0/B1 identity、B2方式、U9a atomic batch、fork capabilityの反例時 |
| package／admission／hostless distribution | `PATTERN K-SUPPLY`。OCI descriptorのdigest/size/media-type、TUFのrollback/freeze protectionだけをC0/C3へ | logical manifest、container、store、trust resultを分離。OCI filesystem/runtime/registry、常設中央backendを採らない | path traversal、symlink、duplicate、oversize、corrupt、unknown、tag差替え、rollback、freeze、offline verify、key失効 | C4採択後に`DELETE-LATER`指定のprobe artifactsだけを`RETIRE`。旧静的link routeはV1互換として`KEEP` | threat model、spec/version、offline topology、key custody、commercial local package要件変更時 |

## 6. 依存／並列の直列点

この表は§3〜§5の各子項目に対する必須の依存／並列欄である。`並列可`はdocsまたは隔離private probe同士だけを指し、
`VSM-A9`前の複数Vism runtime実装を意味しない。

| 子項目 | 必須依存 | 共有writer／event loop／GPU device等の直列点 | 現在の並列範囲 |
|---|---|---|---|
| 単一pass Filter | A4I、A5、対象lane A9 | Host GPU device、render thread、PipelineCache | 既存fixtureのread-only再照合だけ |
| multipass Filter／Texture | A8G0→G1→G2→G3、M4-K1a/K1b、M5中間形式 | Host GPU device、pass scheduler、transient pool、color boundary | A8G0 docs-onlyはP0I docsと並列可 |
| Composite | A8G0、対象lane A9 | 同じpass scheduler、input texture lifetime | Filterとの責任比較だけ |
| LayerSource／Generator | A4I、A5、対象lane A9 | Host GPU device、PipelineCache、registry | 既存A3 fixtureの再照合だけ |
| Parameter provider | A4I、A5、対象lane A9 | evaluation schedule、DataTrack owner | provider-only docs／fixtureだけ |
| Data provider→consumer | B0→B1→B2 | DataTrack identity、consumer connection、Document writer | 高量成果のread-only調査だけ |
| Path2D | SDK-S0I main到達。LANG-TS-F0は本結果を後で消費し、Path2Dの前提ではない | native Path oracle、fixture corpus | A8G0／P0I docsと並列可 |
| Shape2D | Path2D、既存VectorRecipe owner、必要ならD2仕様 | native Path oracle、render lowering | Path2D後のdocs-only |
| Text | shaping／font admission owner、M5 identity整合 | font database、glyph cache、render thread | owner決定docsだけ。corpus probe不可 |
| Instance | M5-P0I | domain identity、prototype owner | P0I docs-onlyはA8G0と並列可 |
| Spatial | P0I、M5 camera／geometry／renderer | GPU device、camera／depth／bounds owner | semantic fixture設計だけ |
| 3D import adapter | P0I、M5 canonical geometry／中間形式 | asset admission、import worker、material owner | invalid corpus設計だけ |
| Field | Field／Collider representation、canonical-space owner | field evaluator、budget | representation docsだけ |
| collision query | collider representation、Host SDF正規化／budget owner | acceleration structure、budget | query oracle設計だけ |
| Simulation | SIM-1→VSM-A6 | StateTrack、Bake scheduler、checkpoint writer | solver比較設計だけ。実行probe不可 |
| TypeScript authoring | SDK-S0I main到達→LANG-TS-F0 | Node toolchain pin、fixture publication | F0単独private harness |
| JavaScript live runtime | 根本マップ§9のPhase C入場5条件すべて、B4、B6、LANG-TS-F0。C2が本probeで、その結果をF1へ渡す | engine event loop、budget、last-good owner | 入場後C2内のengine比較だけ |
| admitted WASM module | 根本マップ§9のPhase C入場5条件すべて、B4、B6を満たしてC2でprobe。C4はC0〜C3後の採否gateでありprobe前提ではない | component store、resource limiter、artifact cache | 入場後JSとは別seat／別probe。製品接続0 |
| Generator／Materialize | B0→B1→B2、U9a相当atomic batch | Document single writer、history、preflight snapshot | docs／negative fixtureだけ |
| External Bridge | 対象appの利用者成果、identity、permission決定 | app session、transport、Host adoption | appごとに別private adapter probe |
| Automation | B2、U9a相当atomic batch、permission／job owner | Document writer、Bake／Export job queue | Bridgeと意味決定を分離 |
| Kit／Preset／typed graph | B0→B1→B2→B2I | identity、cycle検査、Document writer | B0〜B2のdocs fixtureだけ |
| package／admission／hostless distribution | 根本マップ§9のPhase C入場5条件すべて。C0起算、C1/C2、C3、C4の段階依存も越えない | artifact publication、install store、trust result | 入場後Phase C内の隔離spikeだけ |

各probe orderはcandidateごとにthread model、state owner、failure mode、platform条件、固定version/hash、license、
positive／negative fixture、`DELETE-LATER`対象を再掲する。いずれかが空ならdispatchしない。

## 7. probeを発注できる順序

| 順序 | grain | 本書から渡すcandidate／終了条件 | 実装可否 |
|---|---|---|---|
| 1 | `VSM-A4I`再判定 | 新依存なし。既存public façadeとtestkit入口を`REUSE` | 本書とA4Sのmain到達後に再判定 |
| 2 | `VSM-A8G0` | `K-WGPU / K-INDUSTRY`。Host multipass責任と負例をdocs-onlyで閉じる | `READY-SPEC`、runtime不可 |
| 3 | `M5-P0I` | `K-GRAPH`のInstance反例だけ。型／schemaを作らない | `READY-SPEC` |
| 4 | `LANG-TS-F0` | `K-TS`。SDK-S0Iと同じPath2D fixture | SDK-S0I main到達後に仕様化 |
| 5 | `VSM-A5 → B0 → B1 → B2` | engine／containerを選ばずmissing、identity、artifact、connectionを閉じる | 直列docs／fixture |
| 6 | Data量閾値の再判定 | `K-ARROW`は`REJECT / NO-PROBE`。B2が高量の実在成果と測定閾値を固定した場合だけ同じ値列／seek oracleで再入場 | B2後も条件未達なら終了 |
| 7a | M5 3D import probe | `K-GLTF`。Spatial意味と分離し、canonical geometry／中間形式決定後に開始 | P0I、M5中間形式待ち |
| 7b | Field fixture | `K-GRAPH`のFieldsは`PATTERN`だけ。candidate dependencyを入れずrepresentationを決める | Field／Collider owner待ち、docs-only |
| 7c | collision query probe | `K-PHYSICS`のParryだけ。Field意味、Simulation solverと別grain | collider representation待ち |
| 7d | Simulation solver probe | `K-PHYSICS`のRapierだけ。Host StateTrack／Bake ownerを維持 | SIM-1、VSM-A6待ち |
| 8a | `VSM-C0` | container候補を同じlogical fixtureで比較 | 根本マップ§9の5条件を全充足後。B6だけでは入場不可 |
| 8b | `VSM-C1 / C2` | source buildと`K-JS / K-WASM` sandboxを別probeで比較 | B4、B6、C0入場後。製品routeへ未接続 |
| 8c | `VSM-C3` | `K-SUPPLY`をinstall前検査／由来のPATTERNに限定 | C0〜C2後 |
| 8d | `VSM-C4` | C0〜C3結果を採用／縮小／延期／棄却 | C0〜C3後。P0/P1未解決0 |

`VSM-A9`が対象laneの非干渉を証明するまでは、上表を「複数Vism runtime実装の並列発注表」と読まない。
並列に進められるのは、ownerと変更許可面が非重複なdocs／private probeだけである。

## 8. 共通negative oracleとSTOP

次のいずれかで当該probeを停止し、`REUSE → REMAP → REDUCE`へ戻す。

- candidate型、engine handle、scene graph、solver world、Arrow array、glTF modelを公開API、Document、serde、plugin contractへ出す。
- probe専用dependencyをworkspace共通依存または製品binaryへ入れる。
- candidateの内部責任からMotoliiのidentity、owner、permission、package、runtime seatを逆算する。
- golden／acceptance期待値をcandidateに合わせて変更する、またはCPU readback、plugin内resource pool、plugin内色変換を加える。
- 同じ機構classのhelper／adapterをVismごとに複製する。
- probe不合格を`BUILD`許可に読み替える。
- cutover前に旧routeを削除する、または新旧二ownerを恒久併存させる。
- version、commit/hash、license、MSRV、3 OS、security、positive／negative fixtureのいずれかが未確認のまま製品採用する。

新しい一般機構がなお不可避に見える場合だけ、既知実装モデルのFable read-only再調査を一回通す。
それでも回避不能なら本書を黙って拡張せず、利用者例外または新しい正本仕様へ戻す。

## 9. 本書が証明しないこと

- `VSM-A4I / A5 / A9`、`B0〜B6`、`C0〜C4`の完了。
- Vism package、manifest、container、loader、install store、署名、runtime engineの採択。
- Text／Instance／Spatial／Field／Simulationの公開契約または製品runtime。
- candidateの性能、安全性、可搬性。ここで固定したのはprobe入口と棄却条件であり、合格結果ではない。
- 二つ以上のVism実装を同時に開始できること。
