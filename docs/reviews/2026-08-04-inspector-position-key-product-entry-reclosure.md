# Inspector Position Add Key product entry reclosure

状態: **決定 / A `DO`、B local `WAIT_TARGET`**

日付: 2026-08-04

## 1. 利用者出口と現行処分

利用者が選択した通常製品入口は、product-owned `InspectorCandidate` の
**Position 行**である。選択中の対象へ現在のplayhead時刻の Position key を一件だけ
追加し、same-time は無変更、成功時は既存 Undo/Redo/journal 経路で同じ
`KeyframeId` を保つ。

ただし、この選択は durable command を再施工する許可ではない。現行通常 Inspector
branch に Position 行、projection、typed Host intent/queue consumer がないため、
`CU-0A08ITI` を A read-only Position row/projection (`CU-0A08ITIA`) と B typed
Host intent/queue (`CU-0A08ITIB`) に分ける。Aだけは
[direct-promotion contract](2026-08-04-inspector-position-row-direct-promotion-contract.md)で
`DO`、B は local `WAIT_TARGET` のままとする。
current playhead の private producer は成立済みだが、Inspector の consumer には未接続である。
この文書は durable command を再施工せず、既存 `CU-0A08ITI` を A/B の接続境界へ再分類する。

```text
AUTHORITY: user-selected normal entry + U4b-0 closed contract + CU-0A08IT split
INTERNAL TARGET: DocumentWriter::prepare_add_position_key(target, current_playhead)
OWNER: DocumentWriter / Command / existing D2 single writer
WRITE ROUTE: Prepared AddPositionKey -> existing queue/runtime -> Undo / JournalEdit v2
GAP: normal Position-row projection/trigger and typed Inspector intent/queue consumer are absent;
  the existing ProductApp current-playhead carrier is not an Inspector consumer
RESOLUTION ROUTE: close A's `Const(Vec2)` value / tag-only `Keyframes` presence Position projection/row first; retain B as WAIT_TARGET
DISPOSITION: Position row is the selected entry; only A is dispatchable
```

## 2. Exact source, current carrier, and missing targets

`ui/motolii-web/src/candidates/InspectorCandidate.jsx` exports the product-owned
`InspectorCandidate`.  Its `mode === "installed"` mock branch contains a `TRANSFORM` /
`Position` `objectRow`, including the existing `automation-mark` DOM/CSS.  That row depends on
mock `state.automation`, mock callbacks/status, fixed transform literals, and mock key-count
literals.  It is source and visual evidence only; it is not a normal product trigger and its
state or callbacks must not be promoted as product semantics.

The normal product branch is `mode === undefined && inspectorReadModel !== undefined`.  It is
mounted by `ui/motolii-web/src/host/inspector-main.jsx` and currently renders only the panel
head, target identity, and an optional active Effect.  It has no Position row, no Position
projection, and no Position event slot.

The existing private Inspector codec is intentionally opacity-only:
`ui/motolii-web/src/host/inspectorHostCodec.js` and
`crates/motolii-ui/src/inspector_host_runtime.rs` accept `effect-param-gesture` for exact
Opacity `amount`.  The read-model decoder validates transform input but does not output a
Position model.  `DocumentEditRuntime` has no Add Position Key action/request.  More
specifically, `DocumentEditQueue` has no `AddPositionKey` action and `ProductApp` handles every
Inspector terminal as an Opacity `SetEffectParamRequest`.

The current-playhead producer is no longer a missing target.  Commit `75ccd5e7` established the
private `ProductApp::editor_playhead.current`: native ruler press/move/release updates it, and
the same value reaches native Timeline/Stage evaluation.  It is deliberately absent from the
Inspector Host contract: `InspectorHostRuntime::new` and `publish` receive only the Document,
primary, and active Effect; its snapshot has no current-time field, and `inspector-main.jsx`
has no Position row or intent slot.  Thus the carrier may not be replaced by a fixed
`RationalTime::ZERO`, but it also cannot be treated as an already-connected Inspector input.

## 3. Known implementation adoption preflight

```text
MECHANISM CLASS: explicit Position-key command entered from a product Inspector control
KNOWN IMPLEMENTATION SEARCH: InspectorCandidate product closure and installed row; private
  Inspector IPC/inbox; DocumentEditQueue publish cycle; U4b-0 prepare/command/journal; current
  ProductApp editor_playhead producer and its native Timeline/Stage consumers
CANDIDATES: reuse product component owner and row visual source / reuse private Host framing /
  reuse U4b-0 writer prepare / reuse existing current-playhead carrier after a real Inspector
  consumer exists / copy installed mock automation / opacity gesture codec / SetProperty plus
  external allocation / generic keyframe API / fixed ZERO time
ADOPTION ROUTE: REUSE only after the missing normal row/projection and typed Inspector
  intent/queue-consumer targets are closed
REJECTED CANDIDATES: mock callbacks or literals = not product semantics; opacity codec = wrong
  identity/command; SetProperty plus allocation = retired by U4b-0; generic API = over-broad;
  ZERO = fabricated current playhead; treating the current native-only carrier as an Inspector
  consumer = missing projection/intent/queue route
THIN MOTOLII SEAM: a future exact private intent may identify the existing primary and read the
  existing ProductApp current playhead, then call the existing Position-only prepare route
THIN MOTOLII RESIDUAL: none is authorized while the normal row/projection and typed consumer
  are absent
RETIREMENT: no external key allocation plus SetProperty product route
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN (new component, generic keyframe framework, second writer, new playhead shape,
  persistence/API changes, easing/value editing)
```

## 4. Future boundary and negative oracles

After A's normal Position row/projection is accepted (without evaluating or counting Keyframes), and only after B's typed Inspector intent/queue
consumer is separately closed, one bounded implementation may connect:
normal Position control -> typed private Inspector intent -> one queue action ->
`prepare_add_position_key(primary, current_playhead)` -> existing publish cycle.

It must prove: no primary/unsupported Position source/stale or malformed IPC yields zero
mutation; same-time yields no action, revision, Undo, or journal edit; successful Undo/Redo and
journal replay retain the durable command's ID.  The normal product route, not the installed
mock branch, must be the later E2E route.  Human visual inspection remains deferred to the M3
final checklist.

Non-goals: Auto Key, Position value editing, active interval, outgoing interpolation/Easing,
mock branch promotion, `P04-C2`, React-owned Document/selection/playhead/Undo state, public
codec/API, Document schema, journal version, and a new product layout/component.
