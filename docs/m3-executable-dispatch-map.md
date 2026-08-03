# M3 実行可能task地図

状態: **施工前コンパイル正本 / 2026-08-01停止simulation反映**

## 1. 目的

[既知技術採択・並列実装地図](m3-parallel-implementation-map.md)の親・子を、施工可否を判断できる
`exact delta`へ変換する。本書は製品意味や新しいtransport gateを作らない。各子について、現在mainで
すでに成立したもの、実装前に閉じる前提、実装担当が変更できるexact target、利用者に見える出口を分ける。

2026-08-01の4子フローsimulationでは、供給routeが決まっていても次の不足がある子を`READY`と呼ぶと、
実装担当が既存adapterを物理移動したり、存在しないtyped snapshotを新設したりすることが分かった。
したがって「既知解がある」と「いま実装taskを開始できる」を別状態にする。

## 2. Codex readiness判定

```text
compile(child, current_main, active_tasks):
    authority = exact_authority(child)
    outcome   = observable_user_exit(child)
    target    = existing_internal_target(child, current_main)
    writer    = unique_owner(target)
    oracle    = executable_positive_and_negative_oracle(child)

    if outcome is already true on shipped route:
        return DONE or VERIFY_ONLY
    if authority leaves product meaning unresolved:
        return SPEC_ONLY(exact_question, exact_fixture)
    if an OSS/adoption route is selected but unprobed on a required platform:
        return ADOPTION_PROBE(upstream, platform_oracle)
    if target is absent:
        return TARGET_MISSING(exact_required_type_or_callback)
    if dynamic_surface(child) and transition_contract(child) is incomplete:
        return TARGET_MISSING(exact_event_owner_intent_or_stale_rule)
    if target intersects an active task or a shared writer:
        return WAIT_CONFLICT(unique_integration_owner)
    if allowlist crosses more than one contract owner:
        return REDUCE
    if raw runtime evidence is the requested outcome:
        return MEASURE
    if synthetic substitution is forbidden:
        return HUMAN or HARDWARE

    return IMPLEMENT(
        one_contract_boundary,
        exact_allowlist,
        exact_read_set,
        pseudocode,
        positive_oracle,
        negative_oracle,
        user_visible_exit,
        next_handoff,
    )
```

上の`compile()`を規範評価器とする。LLMは図を解釈せず、次の同値なtyped branch listを上から一回だけ評価する。

```text
DispatchState =
  DONE | VERIFY_ONLY | SPEC_ONLY | ADOPTION_PROBE | TARGET_MISSING |
  WAIT_CONFLICT | REDUCE | MEASURE | IMPLEMENT | HUMAN | HARDWARE

TRANSITION_ORDER:
  shipped_exit=true                         -> DONE | VERIFY_ONLY
  product_meaning_closed=false              -> SPEC_ONLY
  selected_route_probe_required=true        -> ADOPTION_PROBE
  exact_existing_target=false               -> TARGET_MISSING
  dynamic_transition_incomplete=true         -> TARGET_MISSING
  active_task_or_shared_writer_conflict=true  -> WAIT_CONFLICT
  one_contract_boundary=false               -> REDUCE
  raw_runtime_evidence_required=true         -> MEASURE
  synthetic_substitution_forbidden=true      -> HUMAN | HARDWARE
  otherwise                                  -> IMPLEMENT

POST_IMPLEMENT:
  review(scope=false OR oracle=false)        -> REJECT_AND_ISOLATE
  review(scope=true AND oracle=true)         -> INTEGRATE_ONCE_BY_OWNER
  integrate -> rerun(USER_VISIBLE_EXIT, shipped_route=true)
```

## 3. 状態語

| 状態 | 開始してよいtask |
|---|---|
| `DONE` | なし。既存実装を再施工しない |
| `VERIFY_ONLY` | 既存routeを変えない拒否試験またはshipped-path確認だけ |
| `SPEC_ONLY` | docs、fixture、拒否表だけ。製品codeを変更しない |
| `ADOPTION_PROBE` | upstream互換・thread・platform・licenseを隔離確認。製品接続しない |
| `TARGET_MISSING` | 実装禁止。必要な既存型・callback・snapshotを一意にして前ownerへ返す |
| `WAIT_CONFLICT` | 共有writerまたはactive taskの完了待ち |
| `IMPLEMENT` | 一契約境界、exact allowlist、正負oracle、利用者出口が閉じた実装 |
| `MEASURE` | 既存routeを変えずraw値を採取する。閾値や新機構を仕様化しない |
| `HUMAN` / `HARDWARE` | synthetic代用禁止の外部審判 |

### 3.1 動的UIのcompile条件

React source assetの存在は外観とinteraction資産の再利用を証明するが、Host接続済みの動的挙動までは証明しない。
動的surfaceを`IMPLEMENT`へ上げるには、固定SHAのReact実コードclosureを読み、component単位で次の遷移票を
既存authorityとcode factから埋める。

```text
DYNAMIC_TRANSITION:
  SOURCE: <FIXED_MOCK SHA or PRODUCT_CLOSURE hash + path + export + hook/state/event/effect/story/test closure>
  EVENT: <pointer | keyboard | Host snapshot | playback tick | reload>
  SEMANTIC_OWNER: <Document | UserSettings | Workspace | ProjectSession | Transient | NONE>
  SNAPSHOT_INPUT: <existing typed projection + revision/generation/epoch>
  LOCAL_PRESENTATION: <hover | focus-visible | composition | popover | NONE>
  INTENT_OUTPUT: <existing typed intent/command | NONE>
  TERMINAL_RULE: <preview-only | one D2 commit | read-only>
  STALE_RULE: <revision/generation/epoch mismatch rejection>
  POSITIVE_ORACLE: <observable transition on product route>
  NEGATIVE_ORACLE: <duplicate owner/write/Undo/stale apply = 0>
```

未移管assetは固定mock SHA、移管済みassetはprovenance manifestの現product closure hashを監査する。
静止画、render結果、DOM snapshot、live DOM、React DevToolsから`EVENT`やstate遷移を補完しない。`SEMANTIC_OWNER`、
`SNAPSHOT_INPUT`、`INTENT_OUTPUT`のどれかをcomponent内stateから推測してはならない。既存targetが無ければ、
そのcomponentは`TARGET_MISSING`のまま前ownerへ返す。drag中の表示は`Transient`またはlocal presentationに留め、
terminal時だけ既存D2へ一回commitする。playback tick、Host reload、WebView再生成はrevision/generation/epochで
古い応答を拒否し、最新Host snapshotから再投影する。

## 4. 33子の現在コンパイル結果

この表の状態は基線`555a9ab5`と2026-08-01 simulation時点である。`IMPLEMENT`への昇格は
implementation ledgerへ一意な`DO`行を追加した時だけ発効する。

| 子 | 現在状態 | exact次task | 通常製品routeの出口 |
|---|---|---|---|
| `P01-C1` | `DONE` | `product_runtime_adapter.rs`の単一`ApplicationHandler<ProductEvent>`と`product_runtime.rs`の単一`run_app`を確認。`raw_input_boundary` 5/5、`u1a1_static_viewport` 3/3 | event-loop/surface ownerが一意 |
| `P01-C2` | `TARGET_MISSING` | 固定sourceの未移管componentをsurface別に一件特定し、実コードからdynamic transitionのowner/input/intent/stale ruleを埋める | product単一owner、mockはconsumer |
| `P01-C3` | `TARGET_MISSING` | Browser以外で欠けるrole別epoch/reload callbackを一件特定 | crash/reload後に同じsnapshotを再投影 |
| `P01-C4` | `SPEC_ONLY` | detach時のWorkspace codecとtop-level再生成境界を一問で固定 | layoutを復元してDocument不変 |
| `P02-C1` | `DONE` | `CU-201T-C`で`CU-201T-S`の`TrimClipIn` / `TrimClipOut`を既存D2へ接続済み。`d1l_writer_prepare` 41/41、`motolii-ui` 170/170、fmt/clippy green | trimがjournal replay可能で1 Undo |
| `P02-C2` | `DONE` | `document_edit_runtime` 33/33、製品Place/Timeline/Inspector/Undo delivery 9/9、writer境界 4/4を再実行。既存routeのみ | 全surfaceが同じpublished snapshotを読む |
| `P02-C3` | `TARGET_MISSING` | selection成立済み部分を除き、essential focusまたはplayhead consumerを一件特定 | UI stateをDocumentへ保存せず一方向publish |
| `P03-C1` | `DONE` | `crates/motolii-ui/tests/timeline_projection.rs::p12_hundred_thousand_keys_cull_to_visible_identity`で100k keyの狭域projectionとtyped identityを確認。renderer本体は変更しない | visible outputがbounded |
| `P03-C2` | `DONE / REDUCE` | `CU-201P-MOVE`として、既知NLE body-drag意味を既存`ProductApp`／`TimelineProjection`／`DocumentWriter::prepare_set_clip_start`へ限定接続。広い`CU-201P` interval gestureは引き続き別target | drag中write 0、release 1 Undo、cancel/stale/invalid 0 |
| `P03-C3` | `TARGET_MISSING` | visible-range consumerとnavigation CommandIdを一つ特定 | selection/focus/playheadが同一projection |
| `P04-C1` | `DONE` | なし。`U4a-1`〜`CU-205E`を再実装しない | first-party parameterの通常編集route |
| `P04-C2` | `TARGET_MISSING` | active interval read model、outgoing Interp D2 command、Host codec、React consumerを前ownerへ分離 | easing変更が1 command / 1 Undo |
| `P04-C3` | `TARGET_MISSING` | `CU-204P`へ渡す実在normal operation source | 実providerの診断を既存Feedbackへ投影 |
| `P05-C1` | `TARGET_MISSING` | 現行Stage表示を除き、off-frame/Stage Viewの未成立targetを一つ特定 | 同じcamera/worldでframe内外を表示 |
| `P05-C2` | `SPEC_ONLY` | camera/object targetと既存D2 commandの写像を一問で固定 | 直接操作が1 gesture / 1 Undo |
| `P06-C1` | `ADOPTION_PROBE` | rfdのmain-thread、cancel、Linux portal、parent windowを確認 | dialogからread-only media probeへ到達 |
| `P06-C2` | `SPEC_ONLY` | 動画配置とSoundtrackを分離し、まず動画placement defaultを固定 | valid confirmだけ1 Undo |
| `P07-C1` | `TARGET_MISSING` | GAP-28の`PlaybackSession`→mixed `AudioProgram`接続。seek-onlyならREDUCE | audio主clockでseek/play/pause |
| `P07-C2` | `WAIT_CONFLICT` | P07-C1、raw measurement、M4 provider後 | deadline時だけ古いpreviewをdrop |
| `P07-C3` | `MEASURE` | 10分実素材のclock/drift/drop raw測定 | 長時間再生の同期証拠 |
| `P08-C1` | `TARGET_MISSING` | Export provider snapshotとproduct source assetを一つずつ特定 | settings/start/progress/cancelが通常面に出る |
| `P08-C2` | `WAIT_CONFLICT` | P08-C1とP12-C1後にatomic artifact E2E | 失敗時partial final 0 |
| `P09-C1` | `SPEC_ONLY` | Delete/Duplicate/RenameとClipboard/Pasteを分離。まず一CommandId family | 頻出操作が1 Undoで成立 |
| `P09-C2` | `SPEC_ONLY` | keymap設定UI範囲とIME審判を分離 | composition中shortcut誤発火0 |
| `P09-C3` | `WAIT_CONFLICT` | P09-C1/C2とP01-C4後。OS menu要件が無ければmuda不採用 | button/keymap/menuが同じintent |
| `P09-C4` | `ADOPTION_PROBE` | AccessKit product dependencyとbounded multi-tree graft | keyboard/AXで同じsemantic identity |
| `P10-C1` | `TARGET_MISSING` | 実在providerごとにactivity snapshotを一つ選ぶ | statusとcancel capabilityを正直に表示 |
| `P10-C2` | `TARGET_MISSING` | raw値を受けるcrate-private typed snapshotが不在。新frameworkを作らずownerを決める | HUD非表示でも制御不変なraw計測 |
| `P10-C3` | `WAIT_CONFLICT` | P07-C2、P10-C2、M4 K1/K7/K8 provider後 | high-load後にlatestへ回復 |
| `P11-C1` | `WAIT_CONFLICT` | P01〜P10/P12のLocal Alpha必要子が通常routeへ統合後 | 起動からExportまで一fixtureで完走 |
| `P11-C2` | `HUMAN` | P11-C1 buildで制作、IME、聴感、keyboard/AX審判 | 教材なしの日常制作 |
| `P11-C3` | `HARDWARE` | 所有Windows/Mac、DPI、NVDA/VoiceOver、配布artifact | Distribution Ready |
| `P12-C1` | `SPEC_ONLY` | Unsaved/Save-As/cancel/failed-save policyとrfd probeを分離 | New/Open/Save/reopenで原本を失わない |

## 5. 現在開始できるtask

`CU-201T-S`の意味閉鎖により開始された`CU-201T-C`は、実装 `a860e10e`、oracle補強 `c2eda847`、targeted test/fmt/clippy greenをもって完了した。
`CU-201N-S`で既存`TimelineKey`/`TimelineBar`だけを候補へ採用し、key優先、stable identity tie-break、transient `RationalTime` threshold、no-snapを固定した。
`P02-C1`、`P03-C1-VERIFY`、P02-C2、P01-C1は既存routeの実装・確認として閉じた。`CU-201P-MOVE`は、既知NLEのbody-drag収束意味を既存native event／Transient projection／single writerへREDUCE接続して閉じた。trim edge、snap threshold、slip/slide/roll/ripple、multi-selectを含む広いPRODUCT `CU-201P`は、[CU-201P target gap observation](reviews/2026-08-03-cu-201p-target-gap-observation.md)どおり`WAIT_TARGET`を維持する。Stage placementのpointer captureを流用しない。

### 5.1 `CU-201T-C` — trim command接続

```text
INPUT:
  CU-201T-S TrimClipIn / TrimClipOut contract
  existing SetClipStart command / Writer / undo / JournalEdit v2 route

IMPLEMENT:
  two explicit command variants and distinct merge properties
  Writer prepare from new left/right edge
  atomic apply / inverse / merge / v2 replay
  exact RationalTime and typed rejection matrix

WRITE:
  crates/motolii-doc/src/command.rs
  crates/motolii-doc/src/lib.rs の DocumentWriter 二入口だけ
  crates/motolii-doc/src/undo.rs
  crates/motolii-doc/tests/ only where the existing command oracle family owns the test
  crates/motolii-ui/src/diagnostic_projection.rs の exhaustive CommandKind label 二armだけ

  lib.rs は INPUT の Writer 接続を所有し、diagnostic_projection.rs は CommandKind 追加で
  compile-required になる既存consumerである。いずれも新しい意味・UI gesture・第二writerを追加しない。

FORBIDDEN:
  generic interval command
  delta payload or live-speed replay recomputation
  schema/journal version/plugin contract change
  snap/ripple/slip/retime/UI gesture

EXIT:
  in/out shrink and extend, inverse, merge, JSON/WAL replay and every reject leave exact state
```

### 5.2 `P03-C1-VERIFY` — bounded Timeline oracle

```text
INPUT:
  existing timeline_projection.rs
  existing native_timeline_renderer.rs

VERIFY ONLY:
  valid Document with 100_000 keys
  one-second visible range projects O(visible) keys
  same display label never replaces LayerId identity

ALLOWLIST:
  Timeline-specific test only

FORBIDDEN:
  renderer rewrite
  product_runtime.rs / document_edit_runtime.rs
  threshold adoption

EXIT:
  `p12_hundred_thousand_keys_cull_to_visible_identity` passes; no renderer/product diff
```

`P03-C1-VERIFY`は製品機能の進捗ではなく、既存routeを再施工しないための確認である。確認済みのため、
製品実装の本線は`CU-201N-S → CU-201P`へ戻る。

## 6. ゴールへ至る依存IR

主担当Codexは視覚的な順序を推測せず、次のedgeだけを依存として扱う。同じ`requires`を持たず、
allowlistとwriterが衝突しないnodeは並列にdispatchできる。
ただし、このIRはreadinessを与えない。各nodeは§4のcompile状態とimplementation ledgerの一意な`DO`を通過するまで
dispatch禁止であり、`requires=[]`は`IMPLEMENT`を意味しない。

```text
NODE CU-201T-S       requires=[]                         emits=[trim_semantics]
NODE CU-201T-C       requires=[trim_semantics]           emits=[trim_d2_command]
NODE P03-C2-TRIM     requires=[trim_d2_command]          emits=[native_trim_gesture]

NODE ACTIVE-INTERVAL requires=[]                         emits=[active_interval_identity]
NODE INTERP-COMMAND  requires=[active_interval_identity] emits=[outgoing_interp_command]
NODE P04-C2-EASING   requires=[outgoing_interp_command]  emits=[easing_edit_route]

NODE P01-RESIDUAL    requires=[]                         emits=[role_host_routes]
NODE SURFACE-JOIN    requires=[role_host_routes]         emits=[shared_surface_snapshot]

NODE RFD-PROBE       requires=[]                         emits=[dialog_adoption]
NODE MEDIA-PLACEMENT requires=[dialog_adoption]          emits=[placed_media]
NODE SAVE-POLICY     requires=[]                         emits=[project_policy]
NODE PROJECT-LIFE    requires=[project_policy]           emits=[project_lifecycle]
NODE TRANSPORT       requires=[placed_media]             emits=[transport_route]
NODE EXPORT-UI       requires=[project_lifecycle]        emits=[export_ui_route]
NODE EXPORT-E2E      requires=[transport_route,export_ui_route] emits=[atomic_export]

NODE LOCAL-ALPHA requires=[native_trim_gesture,easing_edit_route,shared_surface_snapshot,atomic_export]
                 emits=[local_alpha_fixture]
NODE HUMAN-ACCEPT requires=[local_alpha_fixture] emits=[human_acceptance]
NODE DISTRIBUTION requires=[local_alpha_fixture] emits=[hardware_distribution_evidence]
```

## 7. 一つの実装taskの確認項目

次の項目をコード事実で確認できない子は実装担当へ送らない。これはtransportが解釈するschemaではない。

```text
PARENT_ID:
CHILD_ID:
LEGACY_TASK:
USER_VISIBLE_EXIT:

AUTHORITY: <path> SHA256:<hash>
CURRENT_CODE_FACT: <path>:<symbol> :: <one unique fact>
CONTRACT_BOUNDARY: <one owner and one meaning>
EXACT_TARGET: <existing type/function/callback>
DYNAMIC_TRANSITION: <required for dynamic surface; otherwise NONE>
ALLOWLIST: <exact files>
READ_SET: <exact files>

PSEUDOCODE:
  input -> validate -> existing owner call -> publish -> observable result

POSITIVE_ORACLE:
NEGATIVE_ORACLE:
CUTOVER:
RETIREMENT:
STOP:
VALIDATION:
NEXT_HANDOFF:
```

完成とは「担当が何か有用なものを作った」ことではない。`USER_VISIBLE_EXIT`が通常製品routeで観測でき、
負例時にDocument/history/artifactが不変で、次handoffが一つに決まった状態を指す。

## 8. 2026-08-01停止simulationの採否

| flow | 観測 | 採否と地図修正 |
|---|---|---|
| Runtime Seam | `ApplicationHandler`を`product_runtime.rs`へ物理移動しようとした | `REJECT`。一意ownerは物理同居ではない。P01-C1を`VERIFY_ONLY`へ変更 |
| Timeline Residual | 100k key狭域projectionとtyped identityのtestだけが閉じた | 実装本体は再施工せず`VERIFY_ONLY`。停止commitは未採用 |
| Inspector/Easing | active interval、outgoing Interp command、Host codec、React consumerが不在 | 正しい`TARGET_MISSING`。UIから推測しない |
| Raw Telemetry | numeric traceは文字列、renderer statsは局所値でtyped snapshot不在 | 正しい`TARGET_MISSING`。汎用telemetry frameworkを新設しない |

このsimulation以後、親地図の「前段依存を満たせば既定でREADY」という推定は使わない。本書の33行を
実装前コンパイル結果とし、code/authorityの変化で一行ずつ更新する。
