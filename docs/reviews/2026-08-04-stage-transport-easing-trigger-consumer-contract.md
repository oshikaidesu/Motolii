# P04-C2 ACTIVE-INTERVAL Stage transport Easing trigger consumer contract

状態: **決定 / DONE・ACCEPTED（ACTIVE-INTERVAL read-only consumer sub-boundary のみ） / EXTERNAL_GATE_PENDING**

日付: 2026-08-04

## 1. outcome と authority

利用者が許可した成果は、React Easing panel/trigger を将来 nativeization するための既存
product-owned trigger を、通常製品で mount 済みの Stage transport の実在 slot へ read-only に接続することである。
これは [Position active-interval read-model contract](2026-08-04-position-active-interval-read-model-contract.md)
の `ACTIVE-INTERVAL` node を消費可能にする最小 boundary であり、親 `P04-C2`、
`INTERP-COMMAND`、popup は閉じない。

```text
AUTHORITY        user-authorized React Easing nativeization; P04-C2 ACTIVE-INTERVAL read rule;
                 R2B source-asset inventory; existing ordinary Stage transport mount
INTERNAL TARGET  ProductApp private Position active interval -> StageChrome private transport snapshot
OWNER            Document owns keys/name; ProductApp recomputes; StageChrome publishes; React presents
WRITE ROUTE      none
GAP              Stage transport currently has a static disabled placeholder and no activeInterval input
RESOLUTION ROUTE private output snapshot/publish -> direct existing EasingTriggerCandidate reuse
DISPOSITION      ACTIVE-INTERVAL only is IMPLEMENT; P04-C2 parent remains TARGET_MISSING
```

`StageTransportCandidate` is an ordinary product-mounted child WebView surface, and its existing
`interval-easing` button is the exact disabled placeholder beside play, step, and timecode.  The
product-owned `EasingTriggerCandidate` already owns the object/channel/disabled/pressed ARIA
presentation.  They establish a real consumer and layout slot; neither is a semantic, command, or
popup owner.

## 2. known-implementation adoption preflight

```text
MECHANISM CLASS: private Host output projection of an existing strict-interior Position read model
KNOWN IMPLEMENTATION SEARCH: ProductApp current_document/primary/editor_playhead; StageChromeHostRuntime;
  InspectorHostRuntime dynamic publish pattern; StageTransportCandidate ordinary mount and exact slot;
  EasingTriggerCandidate; R2B source-asset inventory; LayerIdTable::display_name
CANDIDATES: existing PositionActiveInterval derivation; existing Stage transport child WebView;
  existing Inspector private snapshot/subscribe/publish pattern; direct React trigger import
ADOPTION ROUTE: REUSE existing product mount, slot, trigger, document display name and read rule;
  ADOPT the narrow private dynamic-publish pattern only
REJECTED CANDIDATES: static placeholder; Inspector normal Position row; mock runtime; generic Host framework;
  generic channel model; popup/native-window route; input/postMessage bridge
THIN MOTOLII SEAM: one Stage-transport-specific private snapshot and its ProductApp reconciliation points
THIN MOTOLII RESIDUAL: Position-only active/null projection and exact read-only lifecycle
RETIREMENT: static Stage transport interval-easing placeholder
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN
```

The Inspector pattern is reused as a local publication pattern, not as a shared runtime or input
protocol.  `EasingTriggerCandidate` is imported directly by `StageChromeCandidate`; its shape is
not copied, reduced, or reimplemented.  `EasingTriggerCandidate.jsx` itself remains unchanged.

## 3. exact private projection and lifecycle

The Stage transport snapshot remains private to `motolii-ui` and has exactly these fields:

```text
{ mode, timecode, barPosition, tempoStatus, qualityStatus, activeInterval }

activeInterval = null
  | { objectName: string, channel: "Position" }
```

The first five fields keep their present string values and behavior. `activeInterval` is the only
new field. It is non-null only when the existing strict-interior Position derivation succeeds and
`current_document.layers.display_name(layer)` supplies the selected layer's actual display name.
Missing/empty display name is `null`, not a fabricated fallback. `channel` is the literal
`"Position"` solely for accessible presentation. Layer/key IDs, times, `Interp`, Document data,
and any generic channel identity do not cross this Host boundary.

`ProductApp` computes this output from only `current_document`, `primary`, and
`editor_playhead.current`, using the exact fail-closed rule in the read-model contract. It publishes
the initial snapshot when the Stage transport Host is created, then reconciles after a change to
one of those three inputs (including Document replacement/reopen and primary/playhead clear).
Layout/bounds updates do not republish it; play, step, timecode, and playback behavior do not
change. Reconciliation emits `null` for every endpoint, absent primary/document/playhead,
non-Position, malformed, unsupported, or non-`Vec2` case.

`StageChromeHostRuntime` owns a Stage-transport-specific private publish method and evaluates it
only in its transport WebView. The transport bridge accepts exactly `{ snapshot, subscribe, publish }`:
`snapshot` returns the full object, `subscribe` re-renders on a publication, and `publish` is
Host-to-WebView output only. The static header bridge remains separate. There is no `postMessage`,
typed input, callback, queue, or generic bridge abstraction.

`stage-transport-main.jsx` reads/subscribes to this exact snapshot. `StageChromeCandidate` replaces
only its existing disabled placeholder with a direct `EasingTriggerCandidate` import and passes
`activeInterval={snapshot.activeInterval}` and `pressed={false}`. The trigger gets no `onClick`,
no handler, and no mutable pressed state. Its existing active/null ARIA object and `Position`
channel presentation is therefore the sole visible effect.

## 4. scope, allowlist, and prohibited work

The future implementation allowlist is exactly:

- `crates/motolii-ui/src/product_runtime.rs`
- `crates/motolii-ui/src/stage_chrome_host_runtime.rs`
- `crates/motolii-ui/src/browser_host_runtime.rs`, only to replace exact generated asset filenames
  in existing `include_bytes!` constants and their matching existing protocol routes
- focused private unit tests in `crates/motolii-ui/src/`
- `ui/motolii-web/src/candidates/StageChromeCandidate.jsx`
- `ui/motolii-web/src/host/stage-transport-main.jsx`
- `ui/motolii-web/src/host/stageHostBridge.js`
- `ui/motolii-web/guard-tests/browser-ownership.test.mjs`
- the affected `ui/motolii-web/generated-host/**` closure: regenerated Stage transport HTML,
  its hashed JS/CSS/import closure, and `asset-manifest.json`

No public API, dependency, Document/schema/journal/history/Undo mutation, command queue, projection
generation, general Host framework, generalized channel, Inspector Add Position Key work, or
modification of the trigger source/CSS is admitted. In particular this boundary must not send or
write an outgoing `Interp` command, open a popup, alter play/step/timecode, or add React/Host input
intent. Generated Host output is a mechanically rebuilt delivery artifact, never new source
authority. `P04-C2-EASING` remains blocked on the existing `INTERP-COMMAND` edge.

## 5. oracle and acceptance classification

`PRIMARY_ORACLE` is a strict-interior active Position interval yielding exactly the selected layer
display name and `{ channel: "Position" }`, with the existing trigger's active ARIA object and
`aria-pressed="false"`. Endpoint/no-primary/no-document/no-playhead/non-Position/unsupported/missing-name
cases yield `activeInterval: null`, disabled trigger, and its existing disabled ARIA text.

Focused Rust tests must prove the private snapshot follows every permitted reconciliation input and
that serialized Document equality, journal/history/Undo state, command queue, and projection
generation are unchanged. They must also prove no publication is caused by layout alone. React
guard tests must prove the direct trigger import, removal of the duplicate placeholder shape, exact
active/null render state, and absence of `onClick`/input bridge. Existing play/step/timecode DOM and
semantics remain unchanged.

`REPO_LANES` after a code wave are focused `motolii-ui` ProductApp/Stage Host unit tests,
`cargo clippy --locked -p motolii-ui --all-targets -- -D warnings`,
`node --test ui/motolii-web/guard-tests/browser-ownership.test.mjs`,
`npm --prefix ui/motolii-web run build:host`, then
`npm --prefix ui/motolii-web run check:host`, `git diff --check`, and `./scripts/check-docs.sh`.
The post-build diff review must accept only the affected Stage transport generated closure and
matching `browser_host_runtime.rs` filename/route replacements; it rejects unrelated generated
assets, manifest entries, or browser routes. `EXTERNAL_GATES`: native visual,
focus, and accessibility verification of the eventual trigger/popup is deferred; repository lanes
do not close it.

## 6. next edge and non-goals

This contract makes the active interval live in normal production only as a read-only consumer; it
does not authorize any edit. The next separate boundary, if independently selected, is the graph's
`INTERP-COMMAND` owner/write-route/admission contract. It must not be inferred from this trigger.

Implementation acceptance is recorded in [Stage transport Easing trigger implementation acceptance](2026-08-04-stage-transport-easing-trigger-implementation-acceptance.md).
