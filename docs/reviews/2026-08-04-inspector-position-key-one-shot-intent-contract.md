# Inspector Position key one-shot intent contract

状態: **決定 / `CU-0A08ITIB` `CONTRACT_CLOSED / DO`**

日付: 2026-08-04

## 1. 閉じる利用者成果と境界

通常製品 Inspector の Position 行に、値表示とは別の小さな key affordance を一つ置く。
一回のactivationは、その時点のRust-owned primaryと
`ProductApp::editor_playhead.current`を使い、既存
`DocumentWriter::prepare_add_position_key`へ一件だけ配送する。`Prepared`だけが既存の
durable commit / Undo / Redo / JournalEdit v2 / publish routeへ入り、`AlreadyPresent`は
revision、Undo、journal、projection generationを変えない。

これはpresentation/read-model boundary Aを閉じた
[`CU-0A08ITIA`](2026-08-04-inspector-position-row-implementation-acceptance.md)の後続Bである。
Position値編集、Auto Key、区間選択、Interp、Easingは同じ粒へ入れない。

```text
AUTHORITY: CU-0A08ITIA acceptance + U4b-0 durable command + P02-C3 playhead acceptance
INTERNAL TARGET: normal Position row -> private one-shot -> ProductApp current primary/time ->
  DocumentEditQueue -> DocumentWriter::prepare_add_position_key
OWNER: InspectorCandidate presentation / Inspector Host admission / ProductApp target+time /
  DocumentEditRuntime single writer / motolii-doc durable command
WRITE ROUTE: Prepared AddPositionKey -> existing durable commit -> existing full publish
GAP: exact one-shot message, separate inbox, queue action, and product affordance were absent
RESOLUTION ROUTE: reuse current Inspector private IPC, current Wake drain, current queue/runtime,
  current editor playhead, and U4b-0 prepare without a general keyframe API
DISPOSITION: WAIT_TARGET is discharged; one bounded implementation is DO
```

## 2. exact private message and React owner

`inspectorHostCodec.js` adds a sibling one-shot sender to the existing returned private sender.
It does not enter or modify the `effect-param-gesture` session. The wire object has exactly these
two fields:

```json
{"kind":"add-position-key","sequence":1}
```

- `kind` is exactly `add-position-key`.
- `sequence` is a positive safe integer in JS and positive `u64` in Rust. It is monotonic for the
  WebView sender lifetime and advances only after `postMessage` succeeds.
- target ID, time, value, property/path, key ID, active-effect identity, session, phase, version,
  direction, role, acknowledgement, and error payload are forbidden.
- Inspector's current protocol has no version envelope. Browser Host's unrelated versioned
  envelope is not copied into this message.

`inspector-main.jsx` passes only the new sender callback to `InspectorCandidate`. The normal
Position row remains the same `objectRow` grammar and does **not** become a row-sized button. Its
third/hint slot gains one `type="button"` affordance adjacent to the Position value. It may reuse
the existing neutral diamond/dot visual primitive from the same source, but it must expose exact
`aria-label="Add Position Key"`, own no pressed/at-key/automation state, and carry no target/time.
The installed mock automation callback, `state.automation`, `AUTO ON/OFF`, key count, and fixed
values remain excluded. If the private callback is absent, no write affordance is rendered.

This is another post-promotion change to the product-owned Inspector source. Append exactly one
`CU-0A08ITIB` entry to `inspectorPostPromotionChanges`, using the current A tail as
`fixedSourceSha256` and the changed component as `currentSha256`. Do not replace the fixed
baseline or unrelated provenance entries.

## 3. Host admission and stale/duplicate semantics

`InspectorHostRuntime` adds a separate `InspectorPositionKeyInbox`; the existing
`InspectorGestureInbox` fields, active phase, latest update, terminal queue, reconcile rule, and
error meanings stay unchanged.

The position inbox owns only `last_sequence` and a bounded FIFO of admitted sequence numbers.
It decodes the exact two-field shape, rejects unknown/missing fields, zero, replayed, duplicate,
or decreasing sequence, and rejects when its bounded capacity is full. Acceptance enqueues once
then invokes the existing Wake callback. `take_add_position_key_intent` pops once. A malformed or
rejected message does not Wake and does not affect opacity gesture state.

Here `stale` means an already admitted or lower sequence in the same Inspector WebView lifetime;
no snapshot generation, target identity, or playhead is invented on the JS wire. ProductApp drains
the position FIFO on `ProductEvent::Wake` before Browser polling and resolves current authority at
dequeue. Two distinct rapid clicks have distinct sequences: the first may prepare and commit; the
second reaches the same current target/time and becomes U4b-0 `AlreadyPresent`. Replaying the same
wire message is rejected in the Host inbox.

## 4. ProductApp, queue, and single-writer route

For each admitted position intent, `ProductApp` performs one synchronous route in the Wake turn:

1. read current `primary`; if absent, consume the intent and enqueue nothing;
2. capture that primary and `editor_playhead.current` into one private
   `AddPositionKeyRequest { target, time }`;
3. push exactly one `DocumentEditAction::AddPositionKey(request)`;
4. immediately call the existing `DocumentEditRuntime::process_next` with the current primary and
   projection generation;
5. adopt `Prepared` through existing `commit_command` and `adopt_full_publish`;
6. on `AlreadyPresent`, publish nothing and leave revision/history/journal/generation unchanged.

`DocumentEditRuntime` confirms `current_primary == Some(request.target)` before prepare. A mismatch
is a stale private queue request and returns no publish. It calls only
`writer.prepare_add_position_key(request.target, request.time)`:

- `Prepared { command, .. }` uses the existing durable commit route;
- `AlreadyPresent` is `Ok(None)` with no allocation or commit;
- missing target, `PositionSourceUnsupported`, and `PositionValueTypeMismatch` are admitted product
  negative cases and finish with zero mutation/publish rather than exiting the app;
- StableId, track, keyframe, payload, journal, or live-apply invariant errors remain typed runtime
  errors and are not silently converted to a no-op.

The queue/runtime does not reimplement Position source inspection, key allocation, interpolation,
Bezier splitting, Undo, or journal encoding. `motolii-doc` is unchanged.

## 5. 既知実装採択preflight

```text
MECHANISM CLASS: explicit one-shot property-key activation through an existing private Inspector Host
KNOWN IMPLEMENTATION SEARCH: current InspectorCandidate/objectRow/automation-mark primitive;
  inspector-main and inspectorHostCodec; InspectorGestureInbox/Wake; Browser one-shot admission as
  comparison only; ProductApp current primary/editor_playhead; DocumentEditQueue/Runtime;
  U4b-0 prepare and durable commit contract; M3 rapid-acceptance prior art's Blender/FCP/Adobe
  converged diamond keyframe affordance
CANDIDATES: reuse current Inspector IPC/Wake/queue/runtime and U4b-0 prepare / sibling sequence-only
  one-shot / opacity gesture phase / Browser versioned envelope / row-sized button / generic key API
ADOPTION ROUTE: REUSE + PATTERN (adjacent key affordance and bounded monotonic one-shot admission)
REJECTED CANDIDATES: opacity gesture phase=wrong lifecycle/identity; Browser envelope=wrong protocol;
  row-sized button=destroys value-row grammar; JS target/time/value=second authority and stale payload;
  SetProperty+allocation=retired by U4b-0; generic key API=boundary expansion
THIN MOTOLII SEAM: sequence-only private message -> separate Host FIFO -> current Rust primary/time ->
  one queue action -> existing Position-only prepare
THIN MOTOLII RESIDUAL: exact message admission, bounded sequence replay rejection, fail-closed product
  negatives, source/provenance and route guards
RETIREMENT: CU-0A08ITIB WAIT_TARGET is retired by this closed contract; no alternate
  allocate+SetProperty or mock automation product route
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN
```

No dependency is added. Browser Host's versioned catalog/session protocol is a comparison, not a
dependency or copied framework.

## 6. implementation allowlist

Production source:

```text
ui/motolii-web/src/candidates/InspectorCandidate.jsx
ui/motolii-web/src/host/inspector-main.jsx
ui/motolii-web/src/host/inspectorHostCodec.js
ui/motolii-web/source-provenance.json
crates/motolii-ui/src/inspector_host_runtime.rs
crates/motolii-ui/src/document_edit_runtime.rs
crates/motolii-ui/src/product_runtime.rs
crates/motolii-ui/src/browser_host_runtime.rs
```

The last Rust file may change only the exact generated asset `include_bytes!` paths, served route
match literals, and route-test expected paths for every content-hashed asset rotated by the same
normal host build. It may not change response behavior, protocol/source/API meaning, or any other
Rust logic. Generated output allowlist is exactly the tracked changes produced by
`npm --prefix ui/motolii-web run build:host` under `ui/motolii-web/generated-host/**`, including
its manifests; it is not source authority. Untracked, manually edited, or build-unrelated output
is out of scope.

### 6.1 measured build conflict and disposition

The first bounded implementation run measured that the normal host build changed the shared JS
chunk from `tokens-CfIyaXn9.js` to `tokens-CcZ3RUC1.js`. Vite therefore rotated all seven JS asset
filenames that import or contain that shared graph (Host, Inspector, Stage Header, Stage Host
Bridge, Stage Transport, Timeline Tools, and tokens), rewrote all five entry HTML files, and
updated both generated manifests. The generated-output clause above already admits that exact
tracked build result, but the former Inspector-only restriction on `browser_host_runtime.rs` left
the other six new asset paths unavailable to `include_bytes!`; the affected Rust compile then
failed on seven missing old paths.

Disposition: keep the single normal build output and update the existing runtime path literals and
their exact route-test expectations for all seven rotated assets. Do not preserve dual old/new
assets, manually copy or rename generated files, edit generated output by hand, or change unrelated
Rust source. This is an allowlist correction for deterministic bundle closure only; it adds no
product behavior, owner, protocol, public API, dependency, or implementation boundary.

Tests/guards:

```text
ui/motolii-web/guard-tests/inspector-host-codec.test.mjs
ui/motolii-web/guard-tests/browser-ownership.test.mjs
crates/motolii-ui/tests/cu110pih_product_inspector_host_projection.rs (only if generated route
  closure cannot be asserted in existing inline tests)
inline tests in inspector_host_runtime.rs, document_edit_runtime.rs, product_runtime.rs
```

No CSS, read-model decoder, `motolii-doc`, `motolii-eval`, Document/schema/journal, public API,
plugin contract, new component, new crate, or dependency may change.

## 7. primary and negative oracles

`PRIMARY_ORACLE`: one normal product Position affordance emits the exact sequence-only message;
Host admits it independently; ProductApp resolves the current primary/current editor playhead;
one queue action produces one U4b-0 command and existing durable/full publish; Undo/Redo retain the
same ID through the already accepted command route.

Required automated negatives:

- JS: unknown callback event, sequence exhaustion, or throwing `postMessage` emits/advances nothing;
- Host: unknown/missing field, wrong kind/type, zero, replay/decreasing sequence, and full FIFO reject
  with no Wake; opacity gesture active/update/terminal state is byte-for-byte semantically unchanged;
- product: no primary, primary mismatch after request capture, unsupported/non-Vec2 Position, missing
  target, and `AlreadyPresent` yield zero Document mutation, revision, Undo, journal, projection
  generation, and publish;
- duplicate raw sequence is rejected; two distinct rapid activations result in at most one durable
  key at the exact time, with the second `AlreadyPresent`;
- current non-zero playhead is used; fixed `RationalTime::ZERO` is forbidden;
- default row remains one row with one adjacent button, not a button-wrapped row, and has no
  automation/key-count/at-key/React selection/playhead/Undo state;
- existing opacity gesture codec and its multi-phase tests remain unchanged and green;
- provenance stays append-only and product runtime imports no mock/legacy source.

`REPO_LANES`: focused Node codec/ownership guards; `npm --prefix ui/motolii-web run build:host`;
`npm --prefix ui/motolii-web run check:host`; focused `motolii-ui` Host/runtime tests; affected-crate
Rust tests and strict clippy; `./scripts/check-docs.sh`; `git diff --check`.

`EXTERNAL_GATES`: fresh different-family read-only diff review before adoption. Native/WebView
human visual, focus, accessibility, and affordance recognizability remain M3 final
`EXTERNAL_GATE_PENDING`; automated green does not close them.

## 8. STOP / non-goals

Stop this implementation if it requires target/time/value/version on the JS message, a public
intent or keyframe API, new state owner, a second writer, a new generic inbox/framework, Position
evaluation/value editing, key count/at-key truth, opacity gesture lifecycle changes, Document or
journal changes, Easing, Auto Key, or a user-visible error round-trip. Findings outside this exact
boundary are recorded and returned to the Motion Authoring Loop; they do not expand B.
