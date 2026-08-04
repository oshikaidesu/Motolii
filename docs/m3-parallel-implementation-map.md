# M3 既知技術採択・並列実装地図

状態: **現行M3実装入口**（2026-08-01）

## 1. この地図が置き換えるもの

本書は、M3を「Motolii独自機構の正しさを粒ごとに証明する計画」から、**採択済みの既知機構を
製品成果へ接続する並列実装計画**へ置き換える。親項目は検索と供給routeの単位、小項目は実装と
oracleの単位である。施工step数では分割しない。

基線`555a9ab5`の[旧粒度化](reviews/2026-07-22-m3-comfortable-use-granulation.md)には152行、うち
`DO / WAIT / SPLIT`が71行ある。そこには実装だけでなく、選定範囲、owner候補、再確認、mirror同期が
別IDとして残り、実装済み`CU-109`を`WAIT`と記すdriftもある。旧IDは本書の来歴・oracle参照へ吸収し、
現行dispatch queueとして使わない。実状態は[implementation ledger](implementation-ledger.md)、製品意味は
[M3仕様](specs/M3-ui-integration.md)、供給routeと並列境界は本書を正とする。

本書は新しいledger、transport schema、機械gateではない。主担当Codexは
`implementation-ledger.md`の「現在の並列レーン」にある`DO`行を施工前に確認する。各実装taskは
親IDと子IDを一つずつ引用する。実装可能な子を同時に走らせる時だけ、互いに
file-disjointな旧IDまたは子IDをledgerの別laneへ`DO`として昇格させる。旧粒度化表の152行を同期し直さない。

## 2. 固定原則

1. Motoliiが所有するのは作品意味、製品policy、admission、acceptance oracle、絶対規律である。
2. 機構は`REUSE / ADOPT / WRAP / PORT / PATTERN / EXTERNAL`から一度選び、子項目が継承する。
3. Motolii codeは既知機構を写す薄いadapter、製品policy、fixtureに限定する。
4. 同じowner、contract、allowlist、oracle内の複数施工stepを別粒にしない。
5. 並列実装は共有writer、event loop、GPU device、bundle publication、永続形式だけを直列点にする。
6. 旧実装の投入工数を維持理由にしない。同じoracleへ通した後、ownerを一度切り替え、旧routeを
   `FROZEN → RETIRE`する。旧新の二重ownerを恒久化しない。
7. 必須oracle、license、platform、security、maintenanceの具体的反証だけが供給routeを再開する。

## 3. 親項目の検索入口

| 親ID | 検索key | 利用者成果 | 採択済み供給route | 主な旧範囲 |
|---|---|---|---|---|
| `M3-P01-SURFACE` | window WebView React winit wry layout dock lifecycle | 正しい製品windowにReact chromeとnative Stage/Timelineが現れ、再生成できる | `ADOPT` React/Vite/winit/wry、`REUSE` product React資産、Taffy `PATTERN` | W0a/W0b、CU-0A/CU-0B、G0-9 |
| `M3-P02-EDIT` | Document D2 command journal Undo snapshot single writer | 一回の操作が一回だけcommitされ、全surfaceへ同じrevisionが届く | `REUSE` motolii-doc/D2/journal/current edit runtime、transactional command `PATTERN` | CU-G03、CU-101〜111、CU-201*-C、CU-401A/B |
| `M3-P03-TIMELINE` | Timeline projection hit-test virtualization selection playhead move trim snap | 高密度Timelineで選択・移動・trim・snap・navigationが連続する | `REUSE` headless Timeline/direct wgpu、`PORT` Vello局所pass、Rerunのwindowing `PATTERN` | U3a、CU-105/106、CU-201、CU-402 |
| `M3-P04-INSPECTOR` | Inspector parameter effect keyframe easing diagnostic Advanced | 選択対象のparameter、key、easing、診断を同じidentityで編集できる | `REUSE` NodeDesc/parameter mapping/React Inspector/Feedback、generated form `PATTERN` | U4a/b/c、CU-202〜208、CU-0A08I/E |
| `M3-P05-STAGE` | Stage Output Frame camera gizmo off-frame pan zoom fit | Stageで結果と枠外を見て、camera/objectを直接操作できる | `REUSE` wgpu renderer/camera contract/display slot、Vello glyph/path局所pass | U1f/U2d、CU-304/305 |
| `M3-P06-MEDIA` | Files Browser Inbox import probe soundtrack | 素材を選び、動画と楽曲を検査・配置できる | `REUSE` motolii-media/Symphonia、rfdを`EVALUATE → ADOPT`、ffmpeg CLI `WRAP` | U6、CU-G04、CU-301〜306のmedia側 |
| `M3-P07-TRANSPORT` | playback seek scrub audio clock preview deadline pressure | 音声主clockを守ってseek・scrub・再生し、重い時も最新結果へ追いつく | `REUSE` motolii-transport/cpal/rubato/render_worker latest-mailbox | D5/U5、CU-209/210/212、CU-5B02 |
| `M3-P08-EXPORT` | Export ffmpeg progress cancel atomic output probe | 通常製品面から書き出し、失敗時に壊れたfinalを残さない | `REUSE` motolii-export/ffmpeg CLI/provider errors | CU-G05、CU-307〜309 |
| `M3-P09-OPERATIONS` | delete duplicate rename clipboard keymap IME accessibility workspace | 頻出操作、入力、copy/paste、panel操作で行き止まらない | `REUSE` InputRouter/keymap/commands、arboard/AccessKitを`EVALUATE → ADOPT`、OS menuは要件成立時だけmuda | U0c/d、U1d/e、CU-G08/10、CU-401〜407 |
| `M3-P10-RECOVERY` | activity telemetry crash reload surface loss HUD readiness | 処理状態が見え、surface/crash後も正本から復帰する | `REUSE` Host snapshots/generation/typed failure/Playwright/testkit | U1b/c/i、CU-204、CU-5A01〜03、CU-5B03/04 |
| `M3-P11-ACCEPTANCE` | Local Alpha E2E human hardware Windows DPI NVDA VoiceOver | 通常routeを人間・実機・全platformで完走する | `EXTERNAL` human/hardware/OS、`REUSE`同一Local Alpha fixture | CU-G06、CU-5A04〜06、CU-601〜605 |
| `M3-P12-PROJECT` | Project New Open Save Save-As Unsaved reopen lock catalog | projectを安全に開閉・保存し、失敗や未保存を失わない。既知のdesktop保存意味論（journal durability-first、reopen耐久同値）を採択して既存ownerに接続 | `REUSE` motolii-doc catalog/lock/session/lifecycle、rfdを`EVALUATE → ADOPT` | CU-G04、CU-301/302/310 |

### 3.1 dispatch状態

本書の子は設計上の施工packageであり、依存充足だけで`READY`へ上げない。exact delta、既存target、
allowlist、正負oracle、利用者出口まで閉じた現在状態は
[M3実行可能発注地図](m3-executable-dispatch-map.md)を正とする。

| 子 | 状態 | 実装前に閉じるもの |
|---|---|---|
| `P02-C1` | `GATED` | `CU-201T-S`等、各command familyの既存意味。未確定familyを同じ実装へ混ぜない |
| `P04-C3` | `GATED` | `CU-204P`へ接続する実在の通常operation source。fixture専用sourceを作らない |
| `P01-C2` | `GATED` | 固定sourceに無いiconを推測・新造しない。既存product assetの範囲は先行可 |
| `P06-C1` | `GATED / FIXED_MAC_GATE_PASS` | 固定Macはrfd parent sheet、event-loop、selection、Cancel、typed media failureを採択probeで確認済み。Linux portalと製品接続は未完了 |
| `P06-C2` | `GATED` | Soundtrackの選択policy。動画配置までを先に`REDUCE`してもよい |
| `P07-C1` | `TARGET_MISSING / PREFLIGHT ONLY` | GAP-28の製品`PlaybackSession`とmixed `AudioProgram`接続。4経路（program construction、session lifetime、current-time handoff、actual typed control source）が閉じるまで実装しない。seek/scrub単独は将来の明示`REDUCE`候補 |
| `P09-C1` | `GATED` | clipboard payload、ID再採番、Pasteの1 Undo意味。未決ならDelete/Duplicate/Renameだけを先行 |
| `P09-C2` | `GATED` | keymap設定UIのv1範囲と実IME審判 |
| `P09-C4` | `GATED` | AccessKit product dependency、bounded tree graft、各OS adapterの採択probe |
| `P10-C3` | `GATED` | M4 K1/K7/K8の実在provider snapshot。M3独自cache/schedulerで代用しない |
| `P11-C2` | `HUMAN` | IME、聴感、日常制作の人間審判 |
| `P11-C3` | `HARDWARE` | 所有するWindows/Mac、DPI、NVDA/VoiceOver、配布artifact |
| `P12-C1` | `SPEC_ONLY` | OpenMode admission / close ordering・失敗投影 / Save-As identity移譲 / rfd接続の4点を残す。健全なwritable projectのUnsaved promptやdirty stateは対象外 |

`GATED`は親全体の停止ではない。また、この表は現在の実装許可を表さない。同じ親でも
[実行可能発注地図](m3-executable-dispatch-map.md)が`IMPLEMENT`へコンパイルし、implementation ledgerへ
一意な`DO`が載った別子、または成果を保つ`REDUCE` sliceだけを並列に進める。

## 4. 子項目

### M3-P01-SURFACE — 製品windowとsurface

#### `P01-C1` runtime integration seam

- **結果**: `product_runtime.rs`を唯一のevent-loop／surface統合点とし、各親は別moduleで実装して一回の合流だけを行う。
- **再利用target**: `crates/motolii-ui/src/product_runtime.rs`、`native_host_layout.rs`、`layout_authority.rs`、`browser_host_runtime.rs`、`inspector_host_runtime.rs`。
- **薄い残余**: revision付きsnapshot、typed intent、layout epoch、GPU device/queue参照の受渡し。
- **依存／並列**: 最初に内部seamを固定。以後P03〜P10のmodule実装は並列、同fileへの合流だけP01 ownerが直列化する。
- **oracle**: 1 top-level wgpu Surface、2 native viewport、opaque child WebView、CPU pixel bridge 0、Document/Undo不変。
- **cutover**: `app.rs`/egui shellを製品routeに戻さずdiagnostic baselineとしてfreeze。重複lifecycle coordinatorをretire。
- **吸収する旧ID**: CU-0B03/04/05、CU-G01/G02の施工分割、各mirror修理。

#### `P01-C2` React product asset closure

- **結果**: Browser、Inspector、Feedback、Stage chrome、Timeline tools、Easing入口、KEYS/LAYERS、iconをproduct packageの一意ownerへ揃える。
- **再利用target**: `ui/motolii-web/src/candidates/`、`patterns/`、`feedback/`、`host/`、DTCG生成CSS。
- **薄い残余**: 未移管assetは固定mock SHA、移管済みassetはprovenance manifestの現product closure hashからcomponent/hook/state/event/effect/story/test closureを列挙し、動的stateをlocal presentationまたは既存Host projection/typed intentへ一件ずつ分類する。React側にselection、playhead、Undo、Documentを置かない。
- **依存／並列**: componentごとにfileが分かれる範囲は並列。Vite bundle/manifest publicationだけ直列。
- **oracle**: mockはproduct export consumer、copy 0、固定DOM/class/stable ID/ARIA/interaction、visual threshold変更0。SOURCE ASSET manifestとDYNAMIC_TRANSITION票を独立reviewで照合し、source外から補完したinteraction/state 0、Host snapshot/playback/reload後のstale apply 0、semantic stateのcomponent-local owner 0。
- **cutover**: `docs/mocks-ui` runtime importとlegacy adapterを禁止し、旧sourceをoracleとしてfreeze。
- **吸収する旧ID**: CU-0A01〜09、CU-0B01/02、CU-203M/P、CU-205B*。

#### `P01-C3` WebView lifecycle and focus

- **結果**: reload/crash/resize/focus/IME後にHost snapshotから同じroleを再投影する。
- **再利用target**: `browser_host_runtime.rs`、`inspector_host_runtime.rs`、`stage_chrome_host_runtime.rs`、`timeline_tools_host_runtime.rs`、wry child WebView lifecycle。
- **薄い残余**: role、instance epoch、sequence、bounded retry、typed failure。
- **依存／並列**: P01-C1後。各island実装は並列、event-loop試験は直列。
- **oracle**: stale epoch拒否、reload write 0、focus往復、100 resize、offline bundle、unbounded retry 0。
- **cutover**: role別の重複retry/focus stateを一つのHost lifecycle policyへ吸収。

#### `P01-C4` workspace layout, dock and detach

- **結果**: panelのresize/open/close/dock/detachをWorkspace ownerで復元する。
- **再利用target**: Taffy、`layout_authority.rs`、`layout_runtime.rs`、`native_host_layout.rs`。
- **薄い残余**: semantic panel role、layout epoch、Workspace codec、top-level移動時のsurface再生成。
- **依存／並列**: P01-C1/P03-C1/P05-C1後。P09-C2と並列、hardware acceptanceはP11。
- **oracle**: corrupt profileは安全reset、Document/Undo/Final不変、DPI移動で同じsnapshot/selection/playhead。
- **cutover**: toolkit tree、絶対px、surface別semantic cloneを保存しない。

### M3-P02-EDIT — single-writer編集背骨

#### `P02-C1` command family completion

- **結果**: move、trim、Delete、Duplicate、Rename、採択範囲のPasteを既存D2 commandへ追加する。
- **再利用target**: `crates/motolii-doc/src/command.rs`、既存Add/Remove/SetProperty/SetClipStart、`document_command_request.rs`。
- **薄い残余**: product-specific validation、ID remint、typed rejection、inverse command。
- **依存／並列**: 永続command fileとjournal fixtureはこの子だけが所有して直列施工。UI、Timeline、Clipboard adapterは完了後に並列。
- **oracle**: replay同値、失敗時Document不変、1 gesture=1 Undo、重複ID 0、旧journal読込維持。
- **cutover**: raw mutation、UI専用command、同義helper、文字列payloadをretire。
- **吸収する旧ID**: CU-201M-S/M-C/T-S/T-C、CU-401A/Bのcommand側、関連docs-only prerequisite。

#### `P02-C2` durable edit runtime simplification

- **結果**: prepare→journal→live apply→publishを既存順序の一経路へ畳み、全consumerが同じpublished snapshotを読む。
- **再利用target**: `document_edit_runtime.rs`、`document_edit_runtime` tests、`PublishedDocument`、D1m/D2。
- **薄い残余**: poison/failure policy、primary reconcile、generation、session path。
- **依存／並列**: P02-C1と同じownerのため直列。P01/P03/P04/P05/P10は凍結済みsnapshot/intentだけを読んで並列。
- **oracle**: Apply/Undo/Redoのjournal順序、same LayerId、publish漏れ0、no-session write 0、panic後正本不変。
- **cutover**: 重複prepared-action、test-only edit route、旧status mirrorをretire。
- **吸収する旧ID**: CU-G03、CU-104/109/110/111と前提選定・mirror粒。

#### `P02-C3` selection, focus and playhead authority

- **結果**: primary selection、essential focus、playheadをProject session/Transientの既決ownerから一方向publishする。
- **再利用target**: `interaction_state.rs`、`timeline_projection.rs`、`DocumentEditRuntime` publish、既存five-layer ownership。
- **薄い残余**: selection-only intent、focus transfer、safe reopen initial position。
- **依存／並列**: P02-C2後。P03-C2/P04-C1/P05-C2が同じread-only projectionを並列消費。
- **oracle**: Document/serde/journal/UndoへUI state 0、stale generation拒否、surfaceごとのclone 0。
- **cutover**: owner採択を繰り返すdocs粒とconsumer-local selection/playhead stateをretire。

### M3-P03-TIMELINE — Timeline表示と操作

#### `P03-C1` windowed projection and renderer

- **結果**: visible range内のbar/key/playheadだけをheadless projectionからdirect wgpu/Vello局所passへ描く。
- **再利用target**: `timeline_projection.rs`、`native_timeline_renderer.rs`、`native_timeline_overlay.wgsl`、Vello、Rerun windowing pattern。
- **薄い残余**: Motolii time/track semantics、bounded AX projection、hit identity。
- **依存／並列**: P01-C1のGPU seam後。P04/P06/P08とfile-disjointで並列。
- **oracle**: 1000/100k itemでvisible work bounded、display label非identity、readback 0、same wgpu device。
- **cutover**: egui Timeline製品route、全item DOM/AX node、重複rendererをretire。
- **吸収する旧ID**: U3a-1/2、CU-105、renderer/owner/adoption docs chain。

#### `P03-C2` move, trim and snap gesture

- **結果**: native drag preview、terminal admission、P02 command commitを一つのgesture pipelineへ接続する。
- **再利用target**: `timeline_tools_host_runtime.rs`、`host_pointer_capture.rs`、`product_runtime_adapter.rs`、P02-C1 commands。
- **薄い残余**: snap targets/priority、cancel causes、preview overlay。
- **依存／並列**: P02-C1/P03-C1後。move/trim/snapは同じgesture ownerなので一子内の施工stepとする。
- **oracle**: drag中write 0、release 1 Undo、Esc/outside/capture-loss 0、duplicate/stale admission 0、random sequence全巻戻し。
- **cutover**: move/trim/snap別coordinator、UI raw mutation、旧CU-201分割列をretire。

#### `P03-C3` navigation, selection and focus

- **結果**: click/keyboard/search/filter/visible range変更が同じprimary/focus/playhead projectionを使う。
- **再利用target**: `timeline_projection.rs` hit-test、P02-C3、`input_router.rs`、`command_registry.rs`。
- **薄い残余**: Timeline navigation commands、hidden selection表示、bounded focus projection。
- **依存／並列**: P02-C3/P03-C1後。P04-C1/P09-C2と凍結intent越しに並列。
- **oracle**: filtered selection保持、same-id no-op、IME中shortcut抑止、display名分岐0。
- **cutover**: Timeline-local selection/focus/playhead正本をretire。

### M3-P04-INSPECTOR — parameter、key、diagnostic

#### `P04-C1` generated parameter and effect route

- **結果**: NodeDesc/ValueTypeからcontrolを生成し、Effect/Definition/Use/Param identityを保ってpreview/commitする。
- **再利用target**: `parameter_control.rs`、`inspector_host_runtime.rs`、`ui/motolii-web/src/candidates/InspectorCandidate.jsx`、first-party catalog。
- **薄い残余**: control mapping、domain validation、gesture identity、typed refusal。
- **依存／並列**: P02-C2/P02-C3後。P03/P05/P06と並列。
- **oracle**: 全first-party保存param対応またはtyped拒否、100 update nonblocking、release 1 Undo、React正本0。
- **cutover**: label/thumbnail推測、custom first-party panel、fixture-only effect branchをretire。
- **吸収する旧ID**: CU-202、CU-205B/T/P/W/E、CU-0A08I。

#### `P04-C2` keyframe and easing connection

- **結果**: key追加・移動と区間easingを既存native Easing core/React triggerへ接続する。
- **再利用target**: 既存keyframe commands、native Easing contract、`EasingTriggerCandidate.jsx`、Timeline key projection。
- **薄い残余**: active parameter/key identity、curve read model、gesture commit。
- **依存／並列**: P04-C1/P03-C1後。P03-C2とは別fileなら並列、同key commandはP02 ownerが統合。
- **oracle**: drag中write 0、release 1、Esc 0、非対象curve不変、threshold/golden変更0。
- **cutover**: mock-only easing state、第二curve owner、旧CU-0A08E/CU-206分割をretire。

#### `P04-C3` diagnostics and Advanced

- **結果**: disabled/read-only/invalid/unsupportedをBrief/Context/Inspect/Assistiveへ投影し、Advanced開閉を意味不変にする。
- **再利用target**: `diagnostic.rs`、`diagnostic_projection.rs`、Feedback product component、Inspector existing surface。
- **薄い残余**: reason→density mapping、copy、source operation callback。
- **依存／並列**: 実在operation source成立時に接続。P04-C1/P09-C1と並列。
- **oracle**: 5 reason×4 density、callback 0 on empty、open/close serialize不変、direct mutation 0。
- **cutover**: diagnostic専用製品route、silent disabled、UI-owned error stateをretire。
- **吸収する旧ID**: CU-203/204/207/208/406。

### M3-P05-STAGE — StageとOutput Frame

#### `P05-C1` Stage View and Output Frame

- **結果**: 同じcamera/worldからframe内外をnative viewportへ描き、pan/zoom/fitはpresentation stateに保つ。
- **再利用target**: wgpu render、display slot、camera contract、`stage_chrome_host_runtime.rs`。
- **薄い残余**: Stage view transform、off-frame scrim、selection hit projection。
- **依存／並列**: P01-C1後。P03/P04/P06/P08と並列。
- **oracle**: Document/Final pixel不変、frame外選択可、UI thread readback 0、preview/export同一評価。
- **cutover**: static preview、diagnostic canvas、第二camera stateをretire。

#### `P05-C2` direct camera/object manipulation

- **結果**: camera/object gizmoをP02 gesture/commandへ接続する。
- **再利用target**: canonical coordinate contract、P02-C1/C3、Stage native hit path。
- **薄い残余**: gizmo projection、camera vs object target classification、workspace Hand/Fit。
- **依存／並列**: P05-C1/P02-C1後。P03-C2とgesture patternを共有するがsurface/fileは分離。
- **oracle**: 1 gesture=1 Undo、DPI非依存、Hand/FitでDocument不変、camera/object混同0。
- **cutover**: React/native二重gizmo、absolute-px parameterをretire。

### M3-P06-MEDIA — 素材

#### `P06-C1` file selection and media exploration

- **結果**: 採択probeを通したnative file dialogから動画/音声を選び、read-only probe/previewする。
- **再利用target**: rfd候補、`crates/motolii-media` probe/decode、ffmpeg CLI、typed diagnostic。
- **薄い残余**: file kind admission、missing/corrupt/unsupported mapping、range presentation。
- **依存／並列**: rfdのevent-loop/thread/platform probeとP01-C2 Browser後。P02/P03/P04/P05/P08と並列。
- **oracle**: UI thread decode 0、browse/rangeでDocument/Undo不変、path/stateをDocumentへ保存しない。
- **cutover**: custom dialog、filesystem watcher正本、fixture-only sourceをretire。

#### `P06-C2` media placement and soundtrack

- **結果**: admitted mediaを既存Clip/TimeMap/audio commandへ一回commitする。
- **再利用target**: P02-C1、motolii-media metadata、motolii-audio/Symphonia、existing audio boundary。
- **薄い残余**: placement defaults、typed asset identity、Soundtrack selection policy。
- **依存／並列**: P06-C1/P02-C1後。動画配置とSoundtrackは異なるcommand targetなら並列。
- **oracle**: valid confirmだけ1 Undo、duplicate/stale拒否、Soundtrack無しでも同じ制作route。
- **cutover**: Inbox第二asset owner、UI-owned import historyをretire。

### M3-P07-TRANSPORT — seek、再生、縮退

#### `P07-C1` transport UI connection

- **結果**: React transport/native scrubから既存Transportへseek/play/pause/stepを送る。
- **再利用target**: `crates/motolii-transport`、cpal audio clock、P02-C3 playhead projection、render_worker。
- **薄い残余**: typed transport intent、latest seek、idle policy。
- **依存／並列**: P03-C2/P06-C2後。P08/P09と並列。
- **oracle**: repaint/vsync暴走でclock不変、latest seek、停止後idle、UIを主clockにしない。
- **cutover**: UI timer/playback state、preview専用clockをretire。

#### `P07-C2` preview deadline and pressure

- **結果**: audio/timeを変えず、deadline超過時だけ古い表示要求を捨て、capacity pressureを別入力にする。
- **再利用target**: `render_worker.rs` latest mailbox、transport generation、M4 admission/provider snapshots。
- **薄い残余**: deadline policy、drop reason、latest result projection。
- **依存／並列**: P07-C1とLocal Alpha raw measurement後。P10 HUDと凍結snapshot越しに並列。
- **oracle**: Final全frame、容量だけでdropしない、old generation表示0、fixed resolution違反0。
- **cutover**: unbounded queue、第二evaluator、HUD-owned controlをretire。

#### `P07-C3` sync and duration acceptance

- **結果**: 10分実素材でaudio clock、frame drop追従、GPU timestamp、seekを測る。
- **再利用target**: existing D5 fixtures、transport render integration、GPU timestamp evidence。
- **薄い残余**: environment manifestとraw measurementだけ。
- **依存／並列**: P07-C1後のMEASURE。実装laneを止めず、P11前に直列判定。
- **oracle**: drift/drop/raw log、未実行hardwareをgreenにしない、閾値は別採択。
- **cutover**: pre-U5 skeletonだけのD5 DONE主張をretire。

### M3-P08-EXPORT — 書き出し

#### `P08-C1` export job UI adapter

- **結果**: settings/start/progress/cancelを既存export providerへ接続する。
- **再利用target**: `crates/motolii-export`、typed error、ffmpeg CLI、Feedback/activity projection。
- **薄い残余**: ExportJob presentation owner、cancel capability、settings validation。書き出しはUI共有deviceでなく既存`new_headless()` deviceを使う。
- **依存／並列**: P01-C2後。P06/P07/P09と並列。
- **oracle**: JobはDocument外、UI closeで結果不変、unsupported cancelを偽装しない。
- **cutover**: UI-owned queue/process supervisor、string errorをretire。

#### `P08-C2` atomic output and end-to-end export

- **結果**: finishを必ず実行し、成功時だけfinal artifactを公開する。
- **再利用target**: existing encoder/export tests、ffprobe、temporary artifact→atomic publish pattern。
- **薄い残余**: output path policy、typed disk/encoder/probe/missing-asset failure。
- **依存／並列**: P08-C1/P12-C1後。P11 Local Alpha前に直列E2E。
- **oracle**: cancel/failureでpartial final 0、Preview/Export同一評価、save/reopen後probe合格。
- **cutover**: CLI-only completion、finishを飛ばすearly return、golden緩和をretire。

### M3-P09-OPERATIONS — 日常操作と入力

#### `P09-C1` essential commands and clipboard

- **結果**: Delete/Duplicate/Rename/Copy/Pasteをstable CommandIdとP02 commandsへ接続する。
- **再利用target**: `command_registry.rs`、InputRouter、P02-C1、arboard。
- **薄い残余**: typed clipboard payload、ID remint、unsupported copy scope。
- **依存／並列**: P02-C1後。Delete/Duplicate/Rename UIとclipboard IOは並列、commit ownerだけP02へ合流。
- **oracle**: Independent remint、1 Paste=1 Undo、unknown/missing plugin typed拒否、clipboard bytes非正本。
- **cutover**: raw JSON/string scan、cross-document意味の先取りをretire。

#### `P09-C2` keymap and IME

- **結果**: versioned keymap、IME gate、focus orderを製品surfaceへ接続する。
- **再利用target**: `keymap.rs`、`keymap_codec.rs`、`input_router.rs`、winit/wry IME/focus。
- **薄い残余**: stable CommandId registry、platform reservation diagnostics、composition suppression。
- **依存／並列**: P01-C3/P03-C3後。keymap settings UIとIME adapterは並列、実機IMEはP11。
- **oracle**: preedit中shortcut 0、候補位置、Enter非奪取、未知CommandId保持。
- **cutover**: raw key/modifier分岐、synthetic-only human PASSをretire。

#### `P09-C3` menu and workspace affordances

- **結果**: 必要なOS menu/context help/panel commandを同じCommandIdへ投影する。
- **再利用target**: command registry、Feedback、必要時だけmuda、P01-C4。
- **薄い残余**: visible availability、help density、panel role command。
- **依存／並列**: P09-C1/C2とP01-C4後。menuが製品要件でなければmudaを追加しない。
- **oracle**: silent disabled 0、menu/keymap/button同じintent、Document mutationはP02だけ。
- **cutover**: UI surface別command implementationをretire。OS menu要件が無い限りmudaは追加しない。

#### `P09-C4` AccessKit product adoption

- **結果**: 現在harness/先例に留まるAccessKitを、boundedなHost semantic projectionとして製品surfaceへ接続する。
- **再利用target**: AccessKit、winit adapter、既存stable ID/role/label/state、Timelineのwindowed projection。
- **薄い残余**: surface別tree rootのgraft、focus/action callback、visible外の要約node。
- **依存／並列**: P01-C1/P02-C3/P03-C1後。採択probeはP09-C2と並列、platform審判はP11。
- **oracle**: 100k描画item≠100k AX node、同じsemantic ID、keyboard-only完走、Document/selectionの第二owner 0。
- **cutover**: canvas全要素node化、React/native二重tree、表示labelをidentityにするrouteを拒否。

### M3-P10-RECOVERY — activity、復旧、計測

#### `P10-C1` typed activity and recovery projection

- **結果**: 実在providerのqueued/running/completed/failed/cancelledとsurface recoveryを共通Feedbackへ投影する。
- **再利用target**: provider snapshots、generation/epoch、`diagnostic_projection.rs`、Feedback。
- **薄い残余**: activity read model、unknown progress、cancel capability。
- **依存／並列**: P04-C3と各provider成立後。providerごとのadapterはfile-disjointで並列。
- **oracle**: UI-owned queue 0、old epoch拒否、reloadでsemantic stateをcacheから復元しない。
- **cutover**: provider別spinner/error owner、偽cancelをretire。

#### `P10-C2` telemetry and resource settings

- **結果**: raw起動/idle/input/drop/scrub/parameter値、budget setting、pressure/readinessを同じtyped snapshotへ出す。
- **再利用target**: current numeric trace、render worker stats、M4 providers、User settings codec。
- **薄い残余**: measurement manifest、reason text/icon、preset mapping。
- **依存／並列**: 各providerと並列でinstrumentation、policy採択はraw measurement後。
- **oracle**: Document/Undo/Final不変、100 update nonblocking、HUD非表示でも制御同一、閾値同時採択0。
- **cutover**: HUD正本、backend free VRAM正本化、手写し計測をretire。

#### `P10-C3` high-load recovery

- **結果**: deadline/pressureを分け、latest previewとtime-local readinessを表示して回復する。
- **再利用target**: P07-C2、M4 K1/K7/K8 provider snapshots、P10-C2。
- **薄い残余**: composition of existing policies and user-facing reason。
- **依存／並列**: Local Alpha measurement後。Distribution Ready hardwareとは別lane。
- **oracle**: latest追従、Final全frame、未取得≠ready、golden/threshold緩和0。
- **cutover**: M3独自cache/scheduler/resource managerを作らない。

### M3-P11-ACCEPTANCE — Local Alphaと配布

#### `P11-C1` Local Alpha normal and negative fixture

- **結果**: 通常製品起動→素材→編集→key/easing→再生→保存/reopen→Exportを同じfixtureで完走する。
- **再利用target**: P01〜P10の製品route、Playwright、Rust integration tests、ffprobe。
- **薄い残余**: 一つのfixture manifestと証跡集約。
- **依存／並列**: 実装親のintegration後。正例と負例のfixture作成は並列、最終runは直列。
- **oracle**: missing/corrupt/cancel/crashで正本不変、CLI/diagnostic代用0、未実装を成功から除外しない。
- **cutover**: 旧粒ごとのE2E証明を本fixtureへ吸収。

#### `P11-C2` human acceptance

- **結果**: 教材なしの制作、長文IME、focus/keyboard/AX、操作摩擦を人間が判定する。
- **再利用target**: 同じLocal Alpha build/fixture、VoiceOver/NVDA/MS-IME。
- **薄い残余**: observed issueの分類だけ。現場で仕様を発明しない。
- **依存／並列**: P11-C1後。Mac/Windowsの所有hardwareごとに並列。
- **oracle**: 実人間・実IME・実screen reader、synthetic eventからの外挿0。

#### `P11-C3` distribution hardware matrix

- **結果**: Windows WebView2/PMv2、Mac、異DPI/monitor/HDR-SDR/penで同一fixtureを閉じる。
- **再利用target**: wry platform routes、WebView2/WKWebView、existing G0-9 hardware matrix。
- **薄い残余**: supported platform statementとraw evidence。
- **依存／並列**: P11-C1後。platformごとに並列、G0-9最終判定だけ直列。
- **oracle**: ProcessFailed、z-order、capture、DPI、IME、AX、preview/export意味同一。所有しないhardwareをPASSにしない。

### M3-P12-PROJECT — project lifecycle

#### `P12-C1` New, Open, Save, Save-As and Unsaved

- **結果**: `P12-C1`は`SPEC_ONLY`として、desktop文書ライフサイクルの既知意味論（`NSDocument` + Auto-save系意味）を受ける。保存耐久はjournal published snapshot routeを採択し、Saveはcheckpointとして扱うがdurabilityの開始点にはしない。
- **再利用target**: motolii-doc lifecycle/catalog/lock、P02-C2 session source、rfd候補。
- **薄い残余**: OpenMode admission、close orderingとin-flight失敗投影、Save-As destinationと新identity/path/lock/session移譲、rfdのcancel/failed-save投影。健全なwritable projectのUnsaved choiceは残件にしない。
- **依存／並列**: policy決定とrfd probe、P01-C1/P02-C2後。P08 exportとは別artifactで並列。
- **oracle**: unknown保持、future/corrupt/lock typed拒否、cancel/失敗時原本不変、durable reopen同値。
- **cutover**: raw path open、lock steal、UI-owned document session、SaveをExportへ兼用するrouteをretire。
- **吸収する旧ID**: CU-G04、CU-301/302/310のproject lifecycle側。

## 5. 並列衝突表

| 共有境界 | 関係する親 | 処分 |
|---|---|---|
| `document_edit_runtime.rs`とjournal順序 | P02、P03、P04、P05、P06、P09 | P02だけがwriterを編集。他親は凍結したrequest/published snapshotを使う。command追加はP02-C1へ集約 |
| `product_runtime.rs` event loop | P01、P03〜P10 | 各親は別moduleで実装し、P01 ownerがwaveごとに一回合流。並行branchから同fileを直接編集しない |
| wgpu Device/Queue/Surface/format | P01、P03、P05、P07 | P01-C1で参照とformatを固定。renderer内部は並列、device再生成policyとframe submissionは直列 |
| wgpu/Vello version closure | P01、P03、P05、P08 | workspaceのwgpu 29 / Vello 0.9を固定。各子でupgradeせず、変更は専用compatibility粒へ分離 |
| React bundle/manifest | P01、P04、P07、P09、P10 | source componentはfile単位で並列。一つのsurface hash chainへin-flight childは一件。Vite build、manifest、provenance publicationは一回に集約 |
| selection/focus/playhead | P02、P03、P04、P05、P09 | P02-C3を唯一ownerにし、全surfaceは同じprojectionを読む。consumer-local state追加を拒否 |
| Document/public API/serde/plugin contract | 全親 | 本地図では変更禁止。必要なら当該子だけ`WAIT / SPEC`へ戻し、他親は継続 |
| OS main thread/focus/IME/z-order/DPI | P01、P06、P09、P11、P12 | code adapterは並列。winit/wry/rfd dialogと実event-loop/hardware acceptanceはplatformごとに直列fixture |
| ffmpeg processとartifact lifecycle | P06、P08 | media decode/probeとexport jobのtyped providerを分離。共通process supervisorを新設しない |
| preview/export GPU lifecycle | P01、P05、P08 | previewは共有UI device、exportは既存headless device。device/queue共有で一本化しない |

## 6. 並列wave

```text
Wave 0: internal seams only
  P01-C1 runtime seam ─┬─ P02-C1 command family ─ P02-C2 durable runtime
                      └─ frozen GPU / snapshot / intent / bundle publication interfaces

Wave 1: file-disjoint implementation
  P01-C2 React assets        P03-C1 Timeline renderer     P04-C1 Inspector
  P05-C1 Stage              P06-C1 media adoption probe  P08-C1 Export UI
  P09-C1 essential ops      P10-C2 instrumentation

Wave 2: interaction and product connections
  P02-C3 selection/focus    P03-C2/C3 Timeline gestures  P04-C2/C3 easing/diagnostics
  P05-C2 direct Stage       P06-C2 placement             P09-C2/C3/C4 input/workspace
  P12-C1 project lifecycle (policy + rfd probe後)

Wave 3: time, recovery and integration
  P07-C1/C2 transport       P08-C2 atomic export         P10-C1/C3 recovery
  P01-C3 lifecycle          P01-C4 dock/detach

Wave 4: acceptance only
  P07-C3 measurement ─ P11-C1 Local Alpha ─┬─ P11-C2 human
                                           └─ P11-C3 hardware/distribution
```

同じwaveのbranchは同じmainから開始できる。ただし上の衝突表にある単一owner fileへ直接合流せず、
凍結interfaceへ実装した後にownerが一回ずつ統合する。hardware/human gateを実装laneのbarrierにせず、
Local AlphaまでとDistribution Readyを分ける。

## 7. 旧M3負債の処分

| 処分 | 対象 | 条件 |
|---|---|---|
| `KEEP` | M2 Document/D2/journal意味、絶対規律、product React source、typed tests、実Mac E2E evidence | 製品意味・oracle・採択routeの現行証拠として参照 |
| `ABSORB` | 旧粒度化の選定範囲、owner候補、再確認、mirror同期、親`SPLIT`行 | 本書の親／子／衝突表へ来歴を写し、dispatch IDとして使わない |
| `REPLACE` | `product_runtime.rs`、`document_edit_runtime.rs`、`app.rs`等に集中した複数責任、surface/operation別の重複coordinator | 子の同一oracle通過後、単一ownerを一回切替。ファイル全消去でなく責任単位で置換 |
| `RETIRE` | egui製品route、mock/legacy runtime import、test-only product substitute、重複decoder/helper、旧新二重state | shipped pathが同じoracleへ合格し、consumer 0と削除範囲が確認できた時だけ |
| `FROZEN` | fixed-Mac harness、historical benchmark、visual generation、旧task evidence | 現行製品codeからimportせず、回帰または来歴参照だけ |

## 8. 供給routeを再開する反証

| route | 再開条件 |
|---|---|
| React/Vite product assets | 固定sourceのDOM/ARIA/interactionを直接移管できない、license/security/update条件が不適合 |
| winit/wry child WebView | supported platformのopaque非重複rectで再現可能なz-order/focus/DPI/crash不適合があり、upstream routeで修正不能 |
| Taffy layout | semantic roleとlogical/physical boundsを表せない、またはWorkspace codecへprivate treeを漏らさないと成立しない |
| direct wgpu + Vello局所pass | 同一device/format/alphaで絶対規律を満たせない、または採択versionのwgpu整合が崩れる |
| Rerun `PATTERN` | Motolii fixtureへ型・state・egui依存を持ち込まないとwindowing/virtualizationを移せない |
| motolii-doc/D2/journal | 既存commandで製品oracleを表せず、公開／永続意味の変更が必要。この場合だけ当該子をSPECへ戻す |
| rfd/arboard/muda | platform、license、sandbox、thread modelが製品条件を落とす。mudaはOS menu要件が無ければ採択しない |
| Symphonia/cpal/rubato/ffmpeg CLI | codec/device/platform、VFR、色、finish、process lifecycleの既存oracleを落とす |
| AccessKit | bounded projectionで必要なAX relation/stateを表せない、または全描画itemのnode化が必要 |

反証が一つもないのに、子ごとにecosystem比較、Fable相談、機構の正しさの再証明を行わない。上記反証が
揃い、既知routeの再写像でも回避不能に見える場合だけ、Fableへ一回の取りこぼし検査を行い、なお未解消なら
modelが新機構を仕様化せず利用者例外へ返す。

## 9. 外部反対側レビュー

2026-08-01にFable 5をread-onlyで呼び、現行spec、ledger、主要runtime、Cargo、referencesと本地図案を
照合した。指摘から、projectとmediaの検索親分離、AccessKitの「既採択」誤認修正、rfd main-thread probe、
export headless device、React surface hash chain、wgpu/Vello version closure、残る人間・hardware・仕様gateを
本文へ採用した。Fableの`WebSearch`は検索結果とsnippetを返したが、`WebFetch`による全文確認まで成立しない
資料が含まれたため、それらは`検索確認`に留めた。外部互換性の根拠は下記一次資料をCodexが別途確認した。
Fableの賛同自体はauthorityまたは実装許可に数えない。

## 10. 一次資料と現行採択

- [wry child WebView + wgpu official example](https://github.com/tauri-apps/wry/blob/6b61fcd58b699323ed16956648c3cf566c5da535/examples/wgpu.rs)
- [winit](https://github.com/rust-windowing/winit)、[wry](https://github.com/tauri-apps/wry)、[WebView2 windowed hosting](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/windowed-vs-visual-hosting)、[WKWebView](https://developer.apple.com/documentation/webkit/wkwebview)
- [React](https://react.dev/)、[Vite](https://vite.dev/)、[Taffy](https://github.com/DioxusLabs/taffy)、[AccessKit](https://github.com/AccessKit/accesskit)
- [wgpu](https://github.com/gfx-rs/wgpu)、[Vello](https://github.com/linebender/vello)、[Rerun](https://github.com/rerun-io/rerun)
- [rfd thread/platform notes](https://docs.rs/rfd/latest/rfd/)、[arboard](https://github.com/1Password/arboard)、[muda](https://github.com/tauri-apps/muda)
- [Symphonia](https://github.com/pdeljanov/Symphonia)、[cpal](https://github.com/RustAudio/cpal)、[rubato](https://github.com/HEnquist/rubato)、[FFmpeg](https://ffmpeg.org/documentation.html)
- [Playwright](https://playwright.dev/docs/intro)

version、license、採択／棄却の詳細は[references](references.md)、surface固有の既知issueと固定commitは
[surface topology決定](reviews/2026-07-21-ui-surface-topology-decision.md)を正とする。本節のlink一覧を
子ごとの再調査入口にしない。
