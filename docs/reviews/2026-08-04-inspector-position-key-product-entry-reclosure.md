# Inspector Position Add Key product entry reclosure

状態: **決定 / local `WAIT_TARGET`**

日付: 2026-08-04

## 1. 利用者出口と現行処分

利用者が選択した通常製品入口は、product-owned `InspectorCandidate` の
**Position 行**である。選択中の対象へ現在のplayhead時刻の Position key を一件だけ
追加し、same-time は無変更、成功時は既存 Undo/Redo/journal 経路で同じ
`KeyframeId` を保つ。

ただし、この選択は durable command を再施工する許可ではない。現行通常 Inspector
branch に Position 行がなく、current playhead を渡す実在 carrier もないため、
`CU-0A08ITI` の Add Position product connection は local `WAIT_TARGET` のままとする。
この文書は既存 ID の状態を再分類するだけで、新しい grain ID や code ticket を作らない。

```text
AUTHORITY: user-selected normal entry + U4b-0 closed contract + CU-0A08IT split
INTERNAL TARGET: DocumentWriter::prepare_add_position_key(target, t)
OWNER: DocumentWriter / Command / existing D2 single writer
WRITE ROUTE: Prepared AddPositionKey -> existing queue/runtime -> Undo / JournalEdit v2
GAP: normal Position-row projection/trigger and current-playhead carrier are absent
RESOLUTION ROUTE: retain CU-0A08ITI WAIT_TARGET; close each missing existing target before code
DISPOSITION: Position row is the selected entry; implementation is not dispatchable
```

## 2. Exact source and missing targets

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
importantly, product placement currently supplies `RationalTime::ZERO`; no implemented current
ProjectSession playhead value/shape/producer exists.  The historical Project-session owner
adoption did not decide or implement that carrier.

## 3. Known implementation adoption preflight

```text
MECHANISM CLASS: explicit Position-key command entered from a product Inspector control
KNOWN IMPLEMENTATION SEARCH: InspectorCandidate product closure and installed row; private
  Inspector IPC/inbox; DocumentEditQueue publish cycle; U4b-0 prepare/command/journal; current
  product evaluation time and ProjectSession decisions
CANDIDATES: reuse product component owner and row visual source / reuse private Host framing /
  reuse U4b-0 writer prepare / copy installed mock automation / opacity gesture codec /
  SetProperty plus external allocation / generic keyframe API / fixed ZERO time
ADOPTION ROUTE: REUSE only after the missing normal row and playhead targets are closed
REJECTED CANDIDATES: mock callbacks or literals = not product semantics; opacity codec = wrong
  identity/command; SetProperty plus allocation = retired by U4b-0; generic API = over-broad;
  ZERO = fabricated current playhead
THIN MOTOLII SEAM: a future exact private intent may identify the existing primary and existing
  current playhead, then call the existing Position-only prepare route
THIN MOTOLII RESIDUAL: none is authorized while those two inputs are absent
RETIREMENT: no external key allocation plus SetProperty product route
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN (new component, generic keyframe framework, second writer, new playhead shape,
  persistence/API changes, easing/value editing)
```

## 4. Future boundary and negative oracles

Only after both missing targets are separately closed may one bounded implementation connect:
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
