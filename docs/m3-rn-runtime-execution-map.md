# M3 RN runtime 実行地図

状態: **現行dispatch地図 / R0-ACCEPT DONE / R1 READY-RECHECK**（2026-08-09）

本書は[現行M3仕様](specs/M3-ui-integration.md)のR0〜R4を、実在target、単一owner、非LLM oracle、依存関係へ落とす。成果waveは利用者出口、nodeは施工境界であり、node数を発注回数や担当LLM数として扱わない。同じowner、同じallowlist、同じoracleで閉じる隣接nodeは一つの短い実装waveにまとめてよい。

旧[既知技術採択地図](m3-parallel-implementation-map.md)と[実行可能task地図](m3-executable-dispatch-map.md)は、既存owner、semantic oracle、失敗例を探す履歴資料である。旧IDや旧rendererを本書へ自動継承しない。

## 1. dispatch規則

実装へ渡せるのは、次の全てが閉じたnodeだけである。

1. 利用者に見える一つの出口と、閉じない範囲
2. current main上の実在targetとsemantic owner
3. 変更可能pathの閉じたallowlist
4. 入力、read projection、terminal intent、single writer、再投影の経路
5. positive oracleと、zero-write／stale／late-event等のnegative oracle
6. 依存nodeと、完了後に再判定する一つのhandoff

不足時は、汎用bridge、第二Host、第二writer、第二GPU deviceを発明せず、`REUSE → REMAP → REDUCE → TARGET_MISSING`の順で戻す。実装候補、test green、外部LLMの`ACCEPT`はmain到達を意味しない。

orderのcompile、実装／調査return、return後の再選定は[発注コンパイルと調査返却loop](reviews/2026-08-07-outcome-order-compilation-and-research-return-loop.md)に従う。`COMPILE`と`TARGET_MISSING`は静的な待ち一覧ではない。前者はclosed orderに不足するexact fact、後者は検索場所、候補、採否、不適合理由、exact gap、再入場条件、安全に継続できるedgeを`RESEARCH_RETURN`として閉じる。各return後に、主担当が本地図とcurrent codeから次nodeを再選定する。

本書の状態語は次の通り。

| 状態 | 意味 |
|---|---|
| `VERIFY_CANDIDATE` | 未統合候補がある。diffとoracleをcurrent mainへ再照合するまで再実装しない |
| `DO` | target、allowlist、oracle、依存が閉じ、着手可能 |
| `COMPILE` | 実在ownerはあるが、closed orderに必要な境界確認が残る。実装禁止 |
| `SPEC_ONLY` | 製品意味の一問が未決。docs、fixture、拒否表だけを閉じる |
| `ADOPTION_PROBE` | platform／upstreamの採択証拠が必要。製品接続しない |
| `TARGET_MISSING` | 必要なtyped targetがcurrent mainにない。前ownerへexact gapを返す |
| `WAIT` | 先行nodeのproduct exit待ち |
| `MEASURE` | 既存routeからraw evidenceだけを採る。方式や閾値を同時採択しない |
| `EXTERNAL_GATE` | 実機または人間受入が必要。synthetic greenで代用しない |
| `DONE` | main到達と所定oracleを確認済み |

## 2. outcome spineと共有所有者

```text
RN shell / native surface input
  → 一つのRN Product Host
  → typed read projection / typed terminal intent
  → 既存DocumentEditRuntime / D2 single writer
  → journal / Undo / published revision
  → RN Inspector + rust-skia Timeline + wgpu/rust-skia Stage
```

| 共有owner | 現行target | 禁止する重複 |
|---|---|---|
| Document mutation | `crates/motolii-ui/src/document_edit_runtime.rs` | RN、renderer、platform adapterからの直接write |
| product semantic oracle | `crates/motolii-ui/src/product_runtime.rs`と`cu110*`／`cu111*` tests | 旧`ProductApp`全体の新RN runtimeへの複製 |
| RN Host ABI | R0で導入する`crates/motolii-ui/src/rn_product_host.rs` | panel別Host、surface別Document runtime |
| RN product source | `ui/motolii-rn/` | 過去UI sourceのmain復元、第二RN app |
| React concept oracle | `ui/motolii-web/src/candidates/`と`source-provenance.json` | DOM、CSS、fixture stateの意味owner化 |
| Stage base | `render_worker.rs`、`display_slot.rs`、既存Stage projection oracle | 第二renderer/device/event loop |
| Timeline meaning | `timeline_projection.rs`、move／trim／snapの既存owner | rust-skia renderer内のDocument意味再実装 |

RN Host ABI、RN app root、同じnative Stage componentへ触れるnodeは直列に合流する。renderer内部がfile-disjointでも、同じHost snapshot schemaを同時変更してはならない。

### 2.1 ordinary editor viability overlay

この表は「普通のdesktop動画編集ソフトとして、利用者がどこまで一貫して作業できるか」をR0〜R4へ重ねた読み取り面である。
新しいfeature list、dispatch node、依存、採用資格を作らず、既存authorityとnodeの現在地だけを利用者outcomeで横断表示する。
baseline調査前の一般機能を推測で埋めず、未抽出部分は`OPEN / ACCEPTED ITEM 0`のまま保つ。

| 利用者outcome | current authority | 対応node | 現在状態 | 閉じる前に必要なこと |
|---|---|---|---|---|
| offline製品appでprojectを開き、同じrevisionを安全に表示する | runtime再基線、R0契約 | `R0-HOST`、`R0-MAC-SEAT`、`R0-STAGE-LIFECYCLE`、`R0-ACCEPT` | `DONE` | [R0 product runtime seat受入](reviews/2026-08-09-m3-r0-product-runtime-seat-acceptance.md)で通常RN artifact、同revision、write 0を確認済み |
| BrowserからRectangleを作り、Stage／Timeline／Inspectorへ同一identityを投影しUndo／Redoする | VS-1、D2、Place、projection oracle | `R1-BROWSER`、`R1-HOST-EDIT`、`R1-STAGE`、`R1-TIMELINE`、`R1-INSPECTOR`、`R1-E2E` | `READY-RECHECK / R0 DONE` | current mainから単一GPU binding、snapshot schema、RN product rootを一契約ずつcompileし、旧route成功を新route合格の代用にしない |
| Position keyを現在時刻へ追加し、Stage gizmoで同じPositionを編集し、Curve／Easingを適用する | Add Position Key決定、`KeyframeId`／`Interp`、Stage／Curve契約 | `R2-FOCUS-PLAYHEAD-AUTHORITY`、`R2-STAGE-GIZMO`、`R2-KEY-COMMAND`、`R2-KEY-UI`、`R2-CURVE-READ`、`R2-CURVE-EDIT` | `OPEN / KNOWN GAPS` | key commandとplayhead consumerは`TARGET_MISSING`。gizmo terminal intentが既存key valueを更新する条件、Auto Key時だけ新keyを作る条件、RN Easing triggerからactive intervalへ至るowner／edgeをUIから発明せず前ownerで閉じる |
| Timelineでseek、selection、move、trim、lane、snapを行う | Timeline projection、move／trim／snap oracle | `R2-SELECTION-AUTHORITY`、`R2-FOCUS-PLAYHEAD-AUTHORITY`、`R2-TL-NAV`、`R2-TL-EDIT` | `COMPILE / TARGET_MISSING` | visible-range consumerとplayhead authorityを特定する。既存move／trimを新rendererへ再実装しない |
| mediaを入れ、保存・再open・再生・書き出しまで完走する | project lifecycle、media、D5、Export、recovery decisions | `R3-PROJECT-*`、`R3-MEDIA-*`、`R3-TRANSPORT-*`、`R3-EXPORT-*`、`R3-RECOVERY`、`R3-E2E` | `OPEN / MIXED` | mixed playback、async export、rename、diagnostic／activity／telemetryの実在target不足をprovider別に返す。seek-only等の独立edgeを過剰直列化しない |
| macOS／Windowsで人が通常利用し、配布artifactとして成立する | platform／human／distribution gate | `R4-MAC-*`、`R4-WIN-*`、`R4-DISTRIBUTION` | `EXTERNAL_GATE_PENDING` | synthetic testや一platformのgreenで代用しない |
| 上記以外で一般的なdesktop NLEに必須のoutcomeを特定する | baseline-required checkpoint | 既存R0〜R4 nodeへの将来mappingだけ。新nodeなし | `OPEN / ACCEPTED ITEM 0 / WEB PROBE PASSED` | fresh non-OpenAI抽出、別family challenge、Codex再照合後にだけ既存nodeへ写す。probeや失敗runのrowを入れない |

Position key→gizmo→Easingの行は、利用者が求める一つの制作背骨を見失わないためのoverlayであり、現在のR2 nodeを一発注へ束ねる許可ではない。
特に`R2-E2E`全体の完了をこの背骨の最初の人間応答地点へ過剰なbarrierにしない。既存node間のexact edgeが閉じた時は、同じ
LayerId／Position channel／KeyframeId／playhead／revisionを追う狭いvertical acceptanceを先に返せるが、そのacceptanceの名前、fixture、
依存はclosed order作成時にcurrent codeから確定する。

### 2.2 current authorityの維持と証拠付き再入場

操作grammar、semantic owner、writer、node境界は、通常施工で使う**反証可能な現行authority**である。
凍結契約ではなく、未検索の可能性や新しい発見を禁止しない。ただし、vendor一社のUI、LLMの提案、mock／probe／test green、
名称差、将来役立つという推測だけでは再決定しない。裁定までcurrent ownerを維持し、第二ownerを仮設しない。

再調査へ入れるのは、次のいずれかをraw evidenceで示せる時に限る。

- current ownerではnamed user outcomeのoperation sequenceまたはrecoveryまで閉じない
- current codeと正本のowner／writer／identityが衝突している
- 複数の成熟した既知実装が、現行境界と両立しない同じ責任分割へ収束している
- zero-write、Undo／Redo、reopen、同一identity等の既存oracleが現行境界では成立しない
- 恒久的な第二ownerを作らず接続できるrouteが現行codeに存在しない

該当時も凍結解除queueや新しい承認工程は作らない。`REUSE → REMAP → REDUCE → RESEARCH_RETURN`で、検索範囲、
反証、exact gap、候補と棄却理由、migration／retirement、positive／negative oracleを返し、採択する変更だけを正本と
decision indexへ同時反映する。これは無闇な追加を防ぐ再入場条件であり、具体的反証に基づく地図改訂を止める規則ではない。

### 2.3 ordinary NLEの「ささくれ」観察inventory

Adobe Premiere、Final Cut Pro、DaVinci Resolve、Avid Media Composerの公式guideを軽く横断すると、派手なeffectよりも、
素材整理、編集関係の保持、欠落時の復旧、仕上げ用metadataに反復する操作体系が見える。この表はそのraw observationを
既存地図へ仮写像したもので、`BASELINE_REQUIRED`採用、dispatch node、schema、UI component、実装順を追加しない。
小さいbuttonやdialogで済むと推測せず、逆に一機能一panelとも推測しない。既存surfaceへ薄く収まるかはowner／write route／oracleを閉じてから決める。

| 観察した操作体系 | current Motolii mapping | 分類 | mappingを閉じる前の焦点 |
|---|---|---|---|
| proxy作成／切替、offline表示、元素材relink | `R3-MEDIA-EXPLORE`、`R3-MEDIA-PLACE`、GAP-3／7、M4-K4 | `ALREADY COVERED / MAP THIN` | source identity、proxy派生物、missing表示、relink候補、reopen／export同一性を分離する。path一致だけを内容同一性にしない |
| folder／bin、検索、label、metadata整理 | `R1-BROWSER`、`R3-MEDIA-EXPLORE` | `UNDERCONSIDERED / MAPPING OPEN` | Document、project catalog、filesystem viewのどれがownerか未確定。検索indexやAI検索をbaselineへ抱き合わせない |
| clip／sequence marker、timecode表示・移動 | `R2-FOCUS-PLAYHEAD-AUTHORITY`、`R2-TL-NAV`、`R2-TL-EDIT` | `UNDERCONSIDERED / MAPPING OPEN` | marker identity、clip-local／sequence-global、snap target、Undo、reopenを決めるまでTimeline UIから意味を作らない |
| video／audio link-unlink、sync保持、channel／role | `R3-MEDIA-PLACE`、`R3-PLAYBACK-AUDIO`、既存audio generalization | `UNIMPLEMENTED / MAP THIN` | linked selection、move／trim時の同期、detach、channel mapping、mixed playbackを一ownerへ閉じ、UI groupingをDocument意味にしない |
| nested sequence／compound clip／precompose相当 | Group／Precompose分離、`CompositionClip`、R2 Timeline | `ALREADY CONSIDERED / PRODUCT ROUTE OPEN` | Groupを暗黙flatten／独立時間化せず、既決のCompositionClip境界へ製品authoringを接続できるか確認する |
| ripple／roll／slip／slide、J/L cut等のtrim family | `R2-TL-EDIT`、既存move／trim oracle | `UNDERCONSIDERED / SPLIT REQUIRED` | 現行body move／edge trimの意味を広げず、各操作のaffected set、sync、cancel、1 Undoを別契約で比較する |
| retime／speed／rate stretch／freeze frame | 既存`TimeMap`、GAP-19、R2 Inspector／Timeline | `KNOWN / PRODUCT AUTHORING UNIMPLEMENTED` | key時刻追従、clip duration、source range、unsupported Black／Loopを既決ownerのまま製品入口へ接続する |
| caption／subtitleの作成、編集、import／export | R3 media／export周辺だけ。semantic ownerなし | `UNDERCONSIDERED / OWNER MISSING` | timed text identity、language／role、style、burn-in／sidecar、Preview／Export一致を比較するまで既存Text layerへ縮退しない |
| multicam、audio stem／高度role routing、transcript-based edit | 既存nodeへ未写像 | `SPECIALIZED / BASELINE UNADJUDICATED` | 一般購入条件か用途別機能かをfresh調査で反証し、比率や搭載有無だけでbaseline認定しない |

raw evidence入口は[Adobe Premiere Help](https://helpx.adobe.com/premiere/desktop.html)、
[Final Cut Pro relink](https://support.apple.com/en-euro/guide/final-cut-pro/ver26f5c8c9/mac)、
[Final Cut Pro compound clips](https://support.apple.com/en-mide/guide/final-cut-pro/verbbd3496b/mac)、
[Final Cut Pro roles](https://support.apple.com/en-ie/guide/final-cut-pro/verb71cbcbe/mac)、
[DaVinci Resolve 20 Editors Guide](https://documents.blackmagicdesign.com/UserManuals/DaVinci-Resolve-20-Editors-Guide.pdf)、
[Avid Media Composer Editing Guide](https://resources.avid.com/SupportFiles/attach/Media_Composer_Editing_Guide_2024.x.pdf)である。
これは網羅調査でも採択票でもなく、fresh non-OpenAI抽出と別family challengeが再検索するquery seedに限る。

## 3. Wave R0 — product runtime seat

R0の利用者出口は、実projectを開くoffline macOS Release appが起動し、resize、focus、unmount/remount後も同じread-only revisionを表示し、Document writeが0であること。

| node | owner / target | 閉じる契約 | 非LLM oracle | 状態 / 次 |
|---|---|---|---|---|
| `R0-HOST` | `motolii-ui`の`rn_product_host`、`document_edit_runtime`、Host FFI | project pathから一つのHostを作り、bounded snapshot、lifecycle intent、typed diagnosticを返す。semantic write 0 | host unit/integration test、unknown/stale/late/double-destroy拒否、既存Place/Undo/Redo test不変 | `DONE` |
| `R0-MAC-SEAT` | Git履歴上の旧macOS app delegate／Xcode／Pods／Hermes bundle | RN window、offline Release bundle、Rust static library、明示project path、fail-closed bootstrap | TypeScript、Jest、ESLint、arm64 Release build、network 0の実起動 | `DONE / HISTORICAL SEAT / SOURCE REMOVED FROM MAIN`。受入証拠はGit履歴に保持し、通常targetへ戻さない |
| `R0-STAGE-LIFECYCLE` | Fabric `MotoliiStage` component viewとHost lifecycle ABI | 一つのnative child viewをregister/mount/resize/focus/unmount/remountし、late eventを拒否する。描画・編集はしない | lifecycle sequence、handle uniqueness、resize/scale、focus、remount同revision、Document JSON不変 | `DONE` |
| `R0-ACCEPT` | 上記三nodeの統合artifact | fixture shellではなく通常RN appでR0出口を確認する | 実`project.json`、Release artifact、process launch、同snapshot、write 0 | `DONE`。根拠は[R0 product runtime seat受入](reviews/2026-08-09-m3-r0-product-runtime-seat-acceptance.md) |

R0はcurrent main上で受入済みである。同じfileに隣接するR1 GPU／draw候補はR0へ含めず、R1のclosed orderと非LLM oracleで別途判定する。

## 4. Wave R1 — VS-1再閉鎖

> **current product source(2026-08-11再訂正)**: RN製品UI接続probeは[`ui/motolii-rn/`](../ui/motolii-rn/README.md)
> (App.tsx 660行: Browser 3タブ、Extensions、panel registry、Timeline 3モード、Fabric spec)、
> rust-skia実証は[`spikes/skia-timeline-probe/`](../spikes/skia-timeline-probe/README.md)にリポ内正本がある。
> R1/R2はこのartifact内で接続し、成功時に状態を`PRODUCT_SOURCE`へ繰り上げる。別targetへのimport／copyを製品化と呼ばない。

R1の利用者出口は、RN BrowserのRectangleから既存D2へ一度だけPlaceし、Stage、Timeline、Inspectorが同じ`LayerId`／revisionを表示し、Undoで三面から消え、Redoで戻ること。

| node | owner / exact source | 閉じる契約 | positive / negative oracle | 状態 / 依存 |
|---|---|---|---|---|
| `R1-SHELL` | `ui/motolii-rn/App.tsx`と既存layout／native registration | 既に同居するBrowser、Stage、Timeline、Inspectorを再構築せず、固定fixtureをHost入力へ置き換える | RN component test、window resize、missing Host fail-closed、registration/publication重複0 | `PRODUCT SOURCE / REUSE`。第二rootを作らない |
| `R1-BROWSER` | `ui/motolii-rn/App.tsx`の既存Browser。`DiscoveryBrowserCandidate.jsx`と`source-provenance.json`はsemantic oracle | Effectsの左レール／結果領域を共通Browser viewとしてMedia／Effects／Createへ使い、サムネイルのみ／カード／リストを同じ表示切替で扱う。single clickは選択だけ、double clickは共通item activationとしてRectangleを正準原点＝Stage中央へ一度投影する。source identity付きterminal intentは一つのRN Product Hostへ接続する。DOM DnDは移さない | 3タブの共通view、3表示切替、single clickでStage不変、double clickで中央投影一回、空identity拒否、Document write 0 | `PARTIAL`。共通Browser view、通常Create catalog、Rectangle選択と中央Rerun Stage activationまで着地。これは旧DnD失敗時の暗黙fallbackではなく利用者が明示した別操作である。現RN appはProduct Host staticlib、明示project path、共通intent bridgeをまだ持たないためD2 terminal intentは未接続。Rectangle専用event busやReact overlayを作らず`R1-SHELL`との共通Host接続へ戻す |
| `R1-HOST-EDIT` | 一つの`rn_product_host`と既存`PlaceRectangleRequest`／`process_next` | RN terminal intentを既存Place、Undo、Redoへ接続し、published snapshotを一度だけ配る | `cu110_product_place_commit`、`cu111_product_undo_redo`、stale/invalid/cancel write 0 | `DONE`(2026-08-09 `37d88be0`: `rn_product_host`接続と`crates/motolii-ui/tests/r1_rn_product_edit_intents.rs`がmain到達) |
| `R1-GPU-BINDING` | R0 native Stage lifecycle、`GpuCtx`、platform Component View | Stage previewとrust-skia surface/contextを一つのwgpu Device/Queueへ束ね、configure/resize/unmount/device-lostを一つのlifecycle ownerで閉じる | device/queue identity、第二device 0、late surface event 0、remount後同revision | `PARTIAL`(2026-08-10実測)。RN Component ViewからCAMetalLayer→wgpu Surface→Host単一`GpuCtx`まで到達済み。未達はrust-skia overlayのraster/composite接続のみ。**未着手として発注すると第二のsurface/device ownerを作る** |
| `R1-STAGE` | 固定Rerun Spatial Viewer、`ui/motolii-rn/`の既存Stage | Document評価のidentity／time／assetをRerun入力へ翻訳し、既存Rerun Stageとrust-skia overlayを同じsurfaceへ載せる。scene／view／camera／pickingは作らない | same LayerId／revision／time、stale拒否、Document write 0、第二device 0 | `COMPILE`。同artifact内のwrapper seam以外を発注しない |
| `R1-TIMELINE` | `timeline_projection.rs`と既存`ProductTimelineProjection` oracle、新rust-skia component | bounded visible read projectionを描画し、offscreen objectを生成しない。編集しない | `cu110pt`系、revision一致、visible bound、resize／zoom read-only | `WAIT(R1-GPU-BINDING)` |
| `R1-INSPECTOR` | `InspectorCandidate.jsx`をconcept oracleとするRN Inspector、既存Inspector projection oracle | primary Rectangleのread-only identity／値を表示する。mock reducerをownerにしない | `cu110pih`／primary-selection consumer、none/stale表示、write 0 | `PARTIAL`(2026-08-10実測)。RN product rootにread-only Inspector panelが実在する。未達はinitial snapshot固定からの更新と値表示。**新設として発注すると二重panelになる** |
| `R1-E2E` | `ui/motolii-rn/`の一つの製品artifact | Browser→D2→三面→Undo→Redoを同一identityで完走する | deterministic sequence、revision列、journal、reopen前提を壊さない、second owner 0 | `WAIT(R1-STAGE..INSPECTOR)` |

並列化できるのは、snapshot schemaと`R1-GPU-BINDING`を凍結した後の`R1-BROWSER`、`R1-STAGE`、`R1-TIMELINE`、`R1-INSPECTOR`のうち、allowlistが交差しない組だけである。Stage/Timeline側でrust-skia backendを個別初期化してはならない。`R1-HOST-EDIT`は一つのHost ABI、`R1-SHELL`はapp root／registration／publicationを所有するため直列、`R1-E2E`は実装nodeではなく統合受入である。

## 5. Wave R2 — 制作操作

R2はR1の同一snapshot／single writerが成立してから、操作familyごとにcompileする。旧P02〜P05のsemantic ownerとoracleは再利用するが、旧WebView、direct-wgpu/Vello、`ProductApp` topologyは再利用しない。巨大な共通gesture frameworkを先に作らない。

| node | owner / exact source | 閉じる契約 | positive / negative oracle | 現在状態 / 依存 |
|---|---|---|---|---|
| `R2-SELECTION-AUTHORITY` | `document_edit_runtime.rs`のpublished primary selection、`input_router.rs`、RN Host snapshot | 成立済みprimary selectionをDocument外のread projectionとして一方向publishする | Document／serde／journalへUI state 0、same-id no-op、stale generation拒否 | `WIRED`(2026-08-10実測)。Stage pointer downがgeometry projection／hit-testを通り、既存selection writerとHost snapshotへ接続済み。**旧`WAIT(R1-E2E)`は実装状態を表していなかった** |
| `R2-FOCUS-PLAYHEAD-AUTHORITY` | essential focus／playhead consumer、gesture epoch、RN Host snapshot | focus、playhead、gesture epochをProjectSession／Transientから一方向publishする | Document／serde／journalへUI state 0、consumer-local owner 0、stale generation拒否 | `TARGET_MISSING`。旧`P02-C3`どおり実在consumerを一件ずつ前ownerで特定する |
| `R2-TL-NAV` | `timeline_projection.rs`、rust-skia Timeline component、`command_registry.rs` | continuous scroll、cursor-centered zoom、fit、seek/scrub、primary/multi/marquee、group展開をbounded viewportで行う | 1000 rich clip／100k keyでvisible work bounded、offscreen draw/AX 0、scrubでDocument write 0 | `COMPILE`。visible-range consumerとnavigation intentを固定後、`WAIT(R2-SELECTION-AUTHORITY, R2-FOCUS-PLAYHEAD-AUTHORITY)` |
| `R2-TL-EDIT` | `timeline_move_gesture.rs`、`SetClipStart`、`TrimClipIn/Out`、既存snap contract | body move、trim、lane move、edge scroll、snap、modifier overrideを一gesture pipelineへ接続する | drag中write 0、release高々1 Undo、Esc/outside/capture-loss/stale 0、random sequence全巻戻し | `COMPILE`。move/trimは再利用、lane commandだけexact target再確認。`WAIT(R2-TL-NAV)` |
| `R2-STAGE-VIEW` | R1 Stage、`render_worker.rs`、`display_slot.rs`、camera contract | frame内外、pan/zoom/fit、grid/guide/path、selected boundsを同じcamera/worldから表示する | Final pixel不変、UI thread readback 0、draw/hit同epoch、Hand/Fit write 0 | `WAIT(R1-E2E)` |
| `R2-STAGE-GIZMO` | canonical display transform、R2-SELECTION-AUTHORITY、既存property command／Inspector gesture oracle | object/group root選択、translate gizmo、snap補助をheadless hit/gestureからterminal property intentへ接続する | drag中write 0、release1 Undo、DPI非依存、100/500 gizmo stress、camera/object混同0 | `SPEC_ONLY`。target classificationとgroup transformの既存command写像を固定後、`WAIT(R2-STAGE-VIEW, R2-SELECTION-AUTHORITY)` |
| `R2-INSPECTOR-EDIT` | `parameter_control.rs`、`inspector_host_runtime.rs`、既存SetProperty／effect-param route | RN numeric/text/scrubをpreview、terminal、cancelへ接続する | first-party保存paramは対応またはtyped拒否、100 update nonblocking、release1 Undo、RN正本0 | `BUILT_UNWIRED`(2026-08-10実測)。依存`R2-SELECTION-AUTHORITY`は充足済み。**`READY_NOW`** |
| `R2-KEY-COMMAND` | `DocKeyframeTrack`、`KeyframeId`、`Interp`、D2 command/journal owner | Position keyのadd/remove/move/value/interpをstable identity付きcommandへする | replay同値、ID再利用0、非対象channel不変、失敗時Document不変 | `PARTIAL`(2026-08-10実測)。`AddPositionKey`／`SetPositionKeyInterp`／`SetPositionKeyValue`がwriter／runtimeに実在する。**旧`TARGET_MISSING`は事実と異なり、ゼロからの再実装を招く最大の乖離だった** |
| `R2-KEY-UI` | RN Inspector key trigger、Timeline key projection、R2-KEY-COMMAND | InspectorのPosition key追加とTimeline key選択／移動を同じidentityへ接続する | button／canvasが同じintent、duplicate time typed拒否、cancel write 0 | `WAIT(R2-KEY-COMMAND, R2-INSPECTOR-EDIT)` |
| `R2-CURVE-READ` | `DocKeyframeTrack`／`Interp` read model、Timeline canvas family、rust-skia Curve component | active channel、interval、key、Bezier handleをbounded projectionとして描画する | same revision/channel、non-finite拒否、offscreen key/AX 0、read-only write 0 | `COMPILE`。active interval projectionを固定後、`WAIT(R2-KEY-COMMAND)` |
| `R2-CURVE-EDIT` | R2-CURVE-READ、R2-KEY-COMMAND、`validate_interp` | key／tangent／linked-broken handle／presetをtransient previewから一terminal commandへ接続する | drag中write 0、release1、Esc0、x1/x2範囲、非対象curve不変 | `WAIT(R2-CURVE-READ, R2-KEY-UI)` |
| `R2-E2E` | 上記nodeの製品artifact | 同じRectangleをStage、Timeline、Inspector、Curveから編集しUndo／Redo／reopenする | same LayerId/channel/revision、gestureごと一Undo、cancel／stale write 0 | `WAIT(R2 nodes)`。統合受入であり実装nodeではない |

R2の並列単位は`Timeline`、`Stage`、`Inspector`、`Curve read`のpresentation moduleである。`R2-SELECTION-AUTHORITY`、`R2-FOCUS-PLAYHEAD-AUTHORITY`、`R2-KEY-COMMAND`、D2 command/journalへの合流は直列とする。各familyは`selection`、`preview`、`terminal`、`cancel`、`stale`を別oracleとして持つ。

## 6. Wave R3 — project workflow

R3は旧P06〜P10／P12の成果を、RN product routeへ接続する。Save、playback、export、activityをUI都合で新ownerへ移さない。R2全体を待つ必要がないnodeもあるが、通常product routeへの統合受入`R3-E2E`はR2-E2E後とする。

### 6.1 project、media、transport、export

| node | owner / exact source | 閉じる契約 | positive / negative oracle | 現在状態 / 依存 |
|---|---|---|---|---|
| `R3-PROJECT-POLICY` | `OpenMode`、`ProjectSession`、journal／lock、既存P12決定 | New/Open/Save/Save-As/closeのOpenMode admission、in-flight ordering、identity/path/lock移譲を固定する | future/corrupt/lock typed拒否、cancel/失敗で原本不変、journal durabilityをSave開始点にしない | `SPEC_ONLY`。残る四問を一問ずつ閉じる |
| `R3-PROJECT-UI` | RN dialogs/forms、`rn_product_host.rs`、ProjectSession、採択済みrfd macOS probe | policyをnative dialogと一つのHost sessionへ接続する。旧egui `shell.rs`をRN route ownerにしない | cancel write 0、ReadOnlyNewer save拒否、durable reopen同値、second session 0 | `WAIT(R3-PROJECT-POLICY)` |
| `R3-MEDIA-EXPLORE` | RN Browser、`motolii-media::probe`／FrameReader、rfd | file selectionからread-only metadata/thumbnail/previewを表示する | UI thread decode 0、missing/corrupt/unsupported typed、browseでDocument/Undo不変 | `COMPILE`。file-kind admissionとRN callbackを固定。`WAIT(R1-E2E)` |
| `R3-MEDIA-PLACE` | admitted media identity、D2 AddTrackItem、audio boundary | video placementとSoundtrackを別terminal intentとして一回commitする | valid confirmだけ1 Undo、duplicate/stale拒否、Soundtrack無しでも制作可能 | `SPEC_ONLY`。動画placement defaultとSoundtrack policyを分離。`WAIT(R3-MEDIA-EXPLORE)` |
| `R3-PLAYBACK-AUDIO` | `PlaybackSession`、`AudioProgram`、`MixProducer`、cpal audio clock | Document由来mixed audioを一つのPlaybackSessionへ供給する | audio主clock、overlap mix、seek非block、UI/vsyncを主clockにしない | `BUILT_UNWIRED`(2026-08-10実測)。`AudioProgram`→`MixProducer`→`PlaybackSession`は実装済みで旧product runtimeから呼ばれている。**未達はRN接続のみ** |
| `R3-TRANSPORT-SEEK` | RN transport、R2 playhead、`Transport`、render worker | audio非依存のseek/scrub/step、latest seek、idleをtyped intent/projectionへ接続する | scrubでDocument write 0、old seek/result表示0、停止後idle | `COMPILE / WAIT(R2-FOCUS-PLAYHEAD-AUTHORITY)`。旧P07のseek-only `REDUCE`を継承 |
| `R3-TRANSPORT-PLAYBACK` | RN transport、`PlaybackSession`、R3-PLAYBACK-AUDIO、render worker | play/pauseとaudio主clock projectionを通常routeへ接続する | repaint暴走でclock不変、pause後idle、UI/vsync主clock化0 | `WAIT(R3-PLAYBACK-AUDIO, R3-TRANSPORT-SEEK)` |
| `R3-PREVIEW-PRESSURE` | render worker latest mailbox、transport generation、将来M4 provider snapshots | deadline超過とcapacity pressureを分け、古いpreviewだけをdropする | Final全frame、容量だけでdrop 0、old generation表示0、fixed resolution違反0 | `WAIT(R3-TRANSPORT-PLAYBACK, provider target)`。M3独自cache/schedulerを作らない |
| `R3-SYNC-MEASURE` | D5 fixtures、実transport/render route、raw environment manifest | 10分実素材のclock/drift/drop/seek/GPU timingを採取する | raw log、未実行hardwareをgreenにしない、閾値同時採択0 | `MEASURE / WAIT(R3-TRANSPORT-PLAYBACK)` |
| `R3-EXPORT-PROVIDER` | `motolii-export::ExportJob`／`ExportReport`／`ExportError`、headless GPU | settings/start/resultをDocument外のjobとして実行し、progress/cancel capabilityを正直に投影する | UI closeでjob意味不変、unsupported cancel偽装0、Preview/Export同一評価 | `TARGET_MISSING`。現行同期APIにproduct async job snapshot／progress／cancel ownerがない |
| `R3-EXPORT-UI` | RN export settings/progress、R3-EXPORT-PROVIDER、diagnostic projection | settings validation、start、known/unknown progress、cancel可否を通常panelへ接続する | invalid start 0、UI-owned queue 0、string error 0 | `WAIT(R3-EXPORT-PROVIDER)` |
| `R3-EXPORT-E2E` | export provider、Project lifecycle、temporary artifact→atomic publish | finish後、成功時だけfinal artifactを公開する | cancel/failure partial final 0、ffprobe、save/reopen後同値、missing asset typed | `WAIT(R3-EXPORT-UI, R3-PROJECT-UI)` |

### 6.2 desktop操作、workspace、a11y、recovery

| node | owner / exact source | 閉じる契約 | positive / negative oracle | 現在状態 / 依存 |
|---|---|---|---|---|
| `R3-OPS-DELETE` | `command_registry.rs`、`document_edit_runtime.rs`、`RemoveTrackItem` | 既存DeleteTargetedItemsをstable CommandIdから一Undoへ接続する | delete 1 Undo、same-id no-op、failure state不変 | `BUILT_UNWIRED`(2026-08-10実測)。依存`R2-SELECTION-AUTHORITY`は充足済み。**`READY_NOW`**。既存D2 routeだけを再利用 |
| `R3-OPS-DUPLICATE` | `duplicate.rs`、`AddTrackItem`、D2 writer | ID remint済みduplicate prepareをstable CommandIdから一Undoへ接続する | subtree Layer/Effect/Key ID remint、failure state不変 | `BUILT_UNWIRED`(2026-08-10実測)。依存`R2-SELECTION-AUTHORITY`は充足済み。**`READY_NOW`**。公開APIは`DocumentWriter::duplicate_track_item`で、commandを返さずその場で適用しUndo登録まで完結する |
| `R3-OPS-RENAME` | layer display-name D2 command owner | Renameをstable CommandIdから一Undoへ接続する | same-id no-op、replay/reopen同値、failure state不変 | `TARGET_MISSING`。現行Command/CommandKindにrename familyがないためwriter ownerへ返す |
| `R3-CLIPBOARD` | OS clipboard adapter候補、D2 command owner | Copy/Pasteのtyped bounded payload、scope、ID remintを固定する | 1 Paste=1 Undo、unknown/missing plugin typed拒否、clipboard bytes非正本 | `SPEC_ONLY / ADOPTION_PROBE`。cross-document意味を先取りせず、macOS/Windows別にadapter採択証拠を取る |
| `R3-KEYMAP-IME` | `keymap.rs`、`keymap_codec.rs`、`input_router.rs`、RN text/native surface adapters | versioned keymap、composition suppression、focus orderを全surfaceで共有する | preedit中shortcut 0、Enter非奪取、未知CommandId保持、canvas capture解除 | `COMPILE / WAIT(R1-E2E)`。実IMEはR4 gate |
| `R3-MENU` | command registry、RN/native menu adapter | button/keymap/menuを同じCommandIdへ投影する | silent disabled 0、menu独自mutation 0 | `WAIT(R3-OPS-DELETE, R3-OPS-DUPLICATE, R3-OPS-RENAME, R3-KEYMAP-IME)`。OS menu要件が無ければ専用libraryを追加しない |
| `R3-WORKSPACE` | RN layout、`layout_authority.rs`／`layout_runtime.rs`の意味oracle、Workspace codec | panel open/close/resize/dock/detachをDocument外で復元する | corrupt profile safe reset、DPI移動で同snapshot、Document/Undo/Final不変 | `SPEC_ONLY`。detach top-levelとsurface再生成境界を固定。`WAIT(R1-E2E)` |
| `R3-A11Y-TREE` | RN accessibility tree、native Timeline/Curve/Stageのbounded semantic projection | canvas surfaceを同じstable identityのbounded AX subtreeへgraftする | 100k item≠100k AX node、keyboard-only route、selection第二owner0 | `ADOPTION_PROBE / WAIT(R2-TL-NAV, R2-STAGE-VIEW, R2-CURVE-READ)`。RN macOS/RNW両adapterを別証拠にする |
| `R3-DIAGNOSTIC` | `diagnostic.rs`、`diagnostic_projection.rs`、将来の実在normal operation source、RN Feedback | 通常操作から到達するdisabled/read-only/invalid/unsupportedをdensity別に投影する | 5 reason×4 density、empty callback 0、UI-owned error state 0 | `TARGET_MISSING`。`CU-204P`再確認どおりproduction source callが0。source成立前にadapter/test injectionを製品接続しない |
| `R3-ACTIVITY` | media/playback/export等の実在provider snapshot、RN Feedback | queued/running/completed/failed/cancelledとcancel capabilityをtyped projectionにする | UI-owned queue 0、old epoch拒否、偽progress/cancel 0 | `TARGET_MISSING`。共通snapshotを先に発明せずproviderごとにtargetを出す |
| `R3-TELEMETRY` | numeric trace、render worker stats、UserSettings codec | raw起動/idle/input/drop/scrub値をcrate-private typed snapshotへ出す | HUD非表示でも制御同一、100 update nonblocking、Document/Final不変 | `TARGET_MISSING`。current raw値を受ける一意なtyped snapshotがない |
| `R3-RECOVERY` | Host epoch、surface lifecycle、provider snapshot、ProjectSession | RN reload/remount、surface/device lost、provider failure後に正本から再投影する | old epoch apply 0、unbounded retry 0、cacheからsemantic state復元0 | `WAIT(R3-PROJECT-UI, R3-MEDIA-EXPLORE, R3-PLAYBACK-AUDIO, R3-EXPORT-PROVIDER)` |
| `R3-E2E` | R2 product artifactと上記R3 node | new/open→media→edit/key/easing→playback→save/reopen→exportを一fixtureで完走する | missing/corrupt/cancel/crashで正本不変、CLI/fixture-only代用0 | `WAIT(R2-E2E, R3 required nodes)`。Local Alpha automated gate |

## 7. Wave R4 — platform、human、distribution

R4は共通semantic completionとplatform acceptanceを分離する。一人／一platformの遅れを、file-disjointな共通実装や他platformの検証停止へ変換しない。

| node / gate | target / 必須証拠 | 状態 / 依存 |
|---|---|---|
| `R4-MAC-ADAPTER` | AppKit/RN macOS Component View、Metal/wgpu surface、capture、focus、DPI、device/surface lost | `COMPILE / WAIT(R3-E2E)`。code adapter |
| `R4-MAC-HUMAN` | 同じLocal Alpha buildで日本語IME、keyboard-only、VoiceOver、drag feel、4K/DPI、長時間制作 | `EXTERNAL_GATE / WAIT(R4-MAC-ADAPTER)` |
| `R4-WIN-ADAPTER` | RNW Component View、Composition/DX12、pointer capture、focus、PMv2 DPI、device lost、native file dialog | `COMPILE`。common Rust/headless contract成立後はMac human gateを待たず進められる。rfd/native dialogはWindows実機のopen/cancel/parent/failure証拠を別に取る |
| `R4-WIN-PRODUCT` | 実Windows RNW artifact、NVDA、MS-IME、mixed DPI/monitor、capture/lost recovery | `EXTERNAL_GATE / WAIT(R4-WIN-ADAPTER, R3-E2E)` |
| `R4-DISTRIBUTION` | arm64/x64 signed/package artifact、offline resources、Rust/Skia/RN notices、crash/reopen | `EXTERNAL_GATE / WAIT(MAC and WIN product gates)` |

Windows common Rust target compileは早期に継続するが、Windows product gateの代用ではない。macOSで成立したsemantic ABIとheadless interactionをWindowsへ共有し、platform adapterだけを別nodeとして検収する。

## 8. 旧M3地図からの再配置

| 旧親 | 現行node | 処分 |
|---|---|---|
| P01 Surface | R0、R1-SHELL、R3-WORKSPACE、R3-RECOVERY、R4 adapters | WebView topologyを捨て、単一RN Host／native component lifecycleのoracleだけ継承 |
| P02 Edit | R1-HOST-EDIT、R2-SELECTION/FOCUS-PLAYHEAD-AUTHORITY、R2-KEY-COMMAND、R3-OPS-* | D2 single writer、journal、published snapshotを継承。command追加はwriter ownerへ集約 |
| P03 Timeline | R1-TIMELINE、R2-TL-NAV、R2-TL-EDIT | direct-wgpu/Velloを捨て、bounded projection／gesture oracleをrust-skiaへ継承 |
| P04 Inspector | R1-INSPECTOR、R2-INSPECTOR-EDIT、R2-KEY/CURVE、R3-DIAGNOSTIC | React conceptと既存parameter意味をRNへ移し、mock stateを捨てる |
| P05 Stage | R1-STAGE、R2-STAGE-VIEW/GIZMO | spatial runtimeをRerunへ委ね、MotoliiはDocument翻訳、D2 terminal、rust-skia authoring overlayだけを維持する |
| P06 Media | R3-MEDIA-EXPLORE/PLACE | rfd probeとmedia coreを継承。Inbox第二ownerを作らない |
| P07 Transport | R3-PLAYBACK-AUDIO/TRANSPORT-SEEK/TRANSPORT-PLAYBACK/PRESSURE/MEASURE | seek-only REDUCEとaudio主clockを分離して継承。GAP-28とM4 provider不在を隠さない |
| P08 Export | R3-EXPORT-* | headless exportを継承。同期APIをprogress/cancel済みと数えない |
| P09 Operations | R3-OPS/CLIPBOARD/KEYMAP/MENU/A11Y | CommandIdとIME gateを継承。winit/wry/AccessKitの採択は現platformで再判定 |
| P10 Recovery | R3-DIAGNOSTIC/ACTIVITY/TELEMETRY/RECOVERY | provider実在前の共通frameworkを禁止 |
| P11 Acceptance | R3-E2E、R4 human/product/distribution | automated、human、hardwareを分離 |
| P12 Project | R3-PROJECT-* | journal durabilityを継承し、残る四問をUIから発明しない |

## 9. 共有境界と依存IR

| 共有境界 | 一意owner | 並列側の制約 |
|---|---|---|
| `document_edit_runtime.rs`／journal／command | D2 writer node | Timeline、Stage、Inspector、media、operationsはtyped requestだけを消費 |
| `rn_product_host.rs` snapshot／intent ABI | Host integration node | panel／renderer別Hostやschema forkを作らない |
| RN app root／bundle／native component registration | RN seat owner | component sourceは並列可、app rootとpublicationはwaveごと一回合流 |
| wgpu Device/Queue/Surface | Stage/platform owner | Timeline/Curve rust-skiaは第二deviceを作らず、surface lifecycle変更は直列 |
| selection／focus／playhead／gesture epoch | R2-SELECTION-AUTHORITY／R2-FOCUS-PLAYHEAD-AUTHORITY | consumer-local semantic ownerを作らない |
| ProjectSession／path／lock | R3-PROJECT | media path、export output、workspace layoutと混同しない |
| platform main thread／IME／AX／DPI | platform adapter | common semantic testsと実hardware gateを分離 |

```text
NODE R0-ACCEPT requires=[R0-HOST,R0-MAC-SEAT,R0-STAGE-LIFECYCLE]

NODE R1-SHELL requires=[R0-ACCEPT]
NODE R1-BROWSER requires=[R0-ACCEPT]
NODE R1-HOST-EDIT requires=[R0-ACCEPT,R1-BROWSER]
NODE R1-GPU-BINDING requires=[R0-ACCEPT]
NODE R1-STAGE requires=[R1-GPU-BINDING]
NODE R1-TIMELINE requires=[R1-GPU-BINDING]
NODE R1-INSPECTOR requires=[R0-ACCEPT]
NODE R1-E2E requires=[R1-SHELL,R1-HOST-EDIT,R1-STAGE,R1-TIMELINE,R1-INSPECTOR]

NODE R2-SELECTION-AUTHORITY requires=[R1-E2E]
NODE R2-FOCUS-PLAYHEAD-AUTHORITY requires=[] state=TARGET_MISSING
NODE R2-TL-NAV requires=[R2-SELECTION-AUTHORITY,R2-FOCUS-PLAYHEAD-AUTHORITY]
NODE R2-TL-EDIT requires=[R2-TL-NAV]
NODE R2-STAGE-VIEW requires=[R1-E2E]
NODE R2-STAGE-GIZMO requires=[R2-STAGE-VIEW,R2-SELECTION-AUTHORITY]
NODE R2-INSPECTOR-EDIT requires=[R2-SELECTION-AUTHORITY]
NODE R2-KEY-COMMAND requires=[] state=TARGET_MISSING
NODE R2-KEY-UI requires=[R2-KEY-COMMAND,R2-INSPECTOR-EDIT]
NODE R2-CURVE-READ requires=[R2-KEY-COMMAND]
NODE R2-CURVE-EDIT requires=[R2-CURVE-READ,R2-KEY-UI]
NODE R2-E2E requires=[R2-TL-EDIT,R2-STAGE-GIZMO,R2-INSPECTOR-EDIT,R2-KEY-UI,R2-CURVE-EDIT]

NODE R3-PROJECT-POLICY requires=[R0-ACCEPT]
NODE R3-PROJECT-UI requires=[R3-PROJECT-POLICY]
NODE R3-MEDIA-EXPLORE requires=[R1-E2E]
NODE R3-MEDIA-PLACE requires=[R3-MEDIA-EXPLORE]
NODE R3-PLAYBACK-AUDIO requires=[] state=TARGET_MISSING
NODE R3-TRANSPORT-SEEK requires=[R2-FOCUS-PLAYHEAD-AUTHORITY]
NODE R3-TRANSPORT-PLAYBACK requires=[R3-PLAYBACK-AUDIO,R3-TRANSPORT-SEEK]
NODE R3-PREVIEW-PRESSURE requires=[R3-TRANSPORT-PLAYBACK,M4-PROVIDER-TARGET]
NODE R3-SYNC-MEASURE requires=[R3-TRANSPORT-PLAYBACK]
NODE R3-EXPORT-PROVIDER requires=[] state=TARGET_MISSING
NODE R3-EXPORT-UI requires=[R3-EXPORT-PROVIDER]
NODE R3-EXPORT-E2E requires=[R3-EXPORT-UI,R3-PROJECT-UI]
NODE R3-OPS-DELETE requires=[R2-SELECTION-AUTHORITY]
NODE R3-OPS-DUPLICATE requires=[R2-SELECTION-AUTHORITY]
NODE R3-OPS-RENAME requires=[] state=TARGET_MISSING
NODE R3-CLIPBOARD requires=[] state=SPEC_ONLY+ADOPTION_PROBE
NODE R3-KEYMAP-IME requires=[R1-E2E]
NODE R3-MENU requires=[R3-OPS-DELETE,R3-OPS-DUPLICATE,R3-OPS-RENAME,R3-KEYMAP-IME]
NODE R3-WORKSPACE requires=[R1-E2E]
NODE R3-A11Y-TREE requires=[R2-TL-NAV,R2-STAGE-VIEW,R2-CURVE-READ]
NODE R3-DIAGNOSTIC requires=[] state=TARGET_MISSING
NODE R3-ACTIVITY requires=[] state=TARGET_MISSING
NODE R3-TELEMETRY requires=[] state=TARGET_MISSING
NODE R3-RECOVERY requires=[R3-PROJECT-UI,R3-MEDIA-EXPLORE,R3-PLAYBACK-AUDIO,R3-EXPORT-PROVIDER]
NODE R3-E2E requires=[R2-E2E,R3-PROJECT-UI,R3-MEDIA-PLACE,R3-TRANSPORT-PLAYBACK,
                      R3-EXPORT-E2E,R3-OPS-DELETE,R3-OPS-DUPLICATE,R3-OPS-RENAME,
                      R3-KEYMAP-IME,R3-WORKSPACE,R3-A11Y-TREE,R3-DIAGNOSTIC,
                      R3-ACTIVITY,R3-TELEMETRY,R3-RECOVERY]

NODE R4-MAC-ADAPTER requires=[R3-E2E]
NODE R4-MAC-HUMAN requires=[R4-MAC-ADAPTER] state=EXTERNAL_GATE
NODE R4-WIN-ADAPTER requires=[COMMON-SEMANTIC-ABI]
NODE R4-WIN-PRODUCT requires=[R4-WIN-ADAPTER,R3-E2E] state=EXTERNAL_GATE
NODE R4-DISTRIBUTION requires=[R4-MAC-HUMAN,R4-WIN-PRODUCT] state=EXTERNAL_GATE
```

このIRは上表の依存欄を列挙した索引であり、readinessを与えない。`M4-PROVIDER-TARGET`と`COMMON-SEMANTIC-ABI`は外部依存名で、M3内の実装nodeではない。edgeが満たされても、各nodeは状態表を上から評価し、`TARGET_MISSING`、`SPEC_ONLY`、`ADOPTION_PROBE`、`MEASURE`、`EXTERNAL_GATE`を`DO`へ読み替えない。

## 10. cutoverと退役

| 新route出口 | 退役できる旧route | まだ退役しないもの |
|---|---|---|
| `R0-ACCEPT` | 重複bootstrap／lifecycle候補 | 旧製品UI、旧VS-1 oracle |
| `R1-E2E` | 旧Browser→Place→三面projectionの通常製品入口 | Timeline/Stage編集、旧semantic tests |
| `R2-E2E` | 旧direct-wgpu/Vello Timelineと旧Stage overlayの製品操作入口 | renderer benchmark、gesture／visual oracle、R3 workflow |
| `R3-E2E` | 旧winit/WebView product workflow入口 | diagnostic fixtures、migration corpus、platform evidence |
| `R4-DISTRIBUTION` | 配布artifactに同梱される旧runtime | 歴史decision、non-LLM oracle、回帰fixture |

退役は新route出口を同じoracleで確認した後、一つのownerが一度だけ行う。新旧へ新機能を二重実装しない。

## 11. 現在の一意な次手

1. `R0-ACCEPT DONE`を基準に、R1のsnapshot schema、GPU binding、product rootをcurrent codeから別契約へcompileする。
2. shared ownerのwrite setを先に固定し、file-disjointなBrowser／Stage／Timeline／Inspectorのread consumerだけを短waveで並列化する。
3. R2以降は本書の`TARGET_MISSING`／`SPEC_ONLY`を前ownerで閉じるまで実装発注へ変換しない。ただしfile-disjointな別nodeのcompileとprobeは止めない。

この地図自体の設計・変更は、M3のauthorityとdispatch境界を決める作業なので外部LLMへ発注しない。実装nodeの発注可否、担当model、並列数は、各nodeをclosed orderへ変換する時点で別途判断する。
