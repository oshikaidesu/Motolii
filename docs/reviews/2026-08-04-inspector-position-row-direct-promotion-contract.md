# Inspector Position row direct-promotion contract

状態: **決定 / `CU-0A08ITIA` のみ `DO`**

日付: 2026-08-04

## 1. 閉じる境界

通常製品 Inspector の `mode === undefined && inspectorReadModel !== undefined` branch に、選択中 target の actual Position を**読むだけ**の一行を出す。これは Add Position Key の入口を準備する presentation/read-model boundary A だけであり、one-shot intent、Host queue、playhead consumer、D2 write は boundary B (`CU-0A08ITIB`) として `WAIT_TARGET` に残す。U4b-0 の最初の durable Add Position Key は `Const(Vec2)` を one-key `Keyframes` へ変換するので、A はその遷移後も同じ Position 行を残す必要がある。

```text
AUTHORITY: Inspector Position entry reclosure + U4b-0 durable command + P02-C3 playhead acceptance
INTERNAL TARGET: decodeInspectorReadModel -> existing InspectorCandidate default branch
OWNER: inspectorReadModelDecoder read-model projection + product-owned InspectorCandidate
WRITE ROUTE: NONE (read-only)
GAP: the default branch has no Position projection or row; the installed row's automation/key literals and callbacks are mock-only
RESOLUTION ROUTE: direct-promote the existing Position objectRow visual grammar in the same source file, supplied only by a narrow typed read model
DISPOSITION: A is one exact implementation DO; B stays WAIT_TARGET
```

## 2. Exact target and supported input

The implementation changes only these product paths and their existing guards:

```text
ui/motolii-web/src/read-model/inspectorReadModelDecoder.js
ui/motolii-web/src/candidates/InspectorCandidate.jsx
docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs
ui/motolii-web/guard-tests/browser-ownership.test.mjs
ui/motolii-web/source-provenance.json
```

`inspectorReadModel.position` is optional and has a closed summary form. The already validated selected item may project exactly `{ kind: "const", x: finite_x, y: finite_y }` from `{ const: { Vec2: [finite_x, finite_y] } }`, or `{ kind: "animated" }` from a `keyframes` tag. The latter is tag-only: the current decoder validates `keyframes` only as an object and does not validate/evaluate its contents, values, times, IDs, interpolation, or count. It therefore communicates animated/keyframed presence only and exposes no `DocParam`, Document, target ID, playhead, evaluated value, or key count. Missing `transform`/`position` preserves the existing valid target projection and omits this row. A non-`Vec2` `Const` `DocValue` and every present unsupported `DocParam` shape (`data`, `vec2_axes`, `look_at`, `follow`) are decoder rejection cases, not fallback display states. The Position key count is omitted for both summary variants.

The existing `objectRow` visual grammar is reused in **the same `InspectorCandidate.jsx` source**: one existing helper/JSX grammar serves the installed row and the default row. The default-row call uses `Position` plus either projected X/Y values for `kind: "const"` or the non-numeric `animated` presence label for `kind: "animated"`; neither form has key text. It must not duplicate the row or introduce a new component, component family, layout slot, or framework. The automation button, `ObjectAutoHint`, `AUTO ON/OFF`, all fixed transform/key literals, `state.automation`, and every mock callback/status mutation are excluded from the default-row call; they are not promoted assets. The resulting row has no event handler, focusable control, `postMessage`, codec sender, local semantic state, or write path.

## 3. Source-asset promotion and guard contract

This is a post-promotion change to the product-owned Inspector asset. The implementation must append one `CU-0A08ITIA` entry to the existing `inspectorPostPromotionChanges` provenance chain, whose `fixedSourceSha256` is the current chain tail and whose `currentSha256` is the changed `InspectorCandidate.jsx`. It must update the existing append-only-chain guard expectation rather than replace the fixed-source baseline, copy the component, or alter unrelated provenance entries.

The existing AST/product guard is extended to prove all of the following:

- the default branch consumes only the closed `inspectorReadModel.position` summary: X/Y for `kind: "const"`, or the literal non-numeric `animated` presence for `kind: "animated"`;
- the exact default branch uses the same existing object-row grammar rather than a copied JSX row;
- a valid finite `Const(Vec2)` produces the Position/X/Y row, while a validated `keyframes` tag produces the Position/`animated` row, both with no key-count or automation text;
- non-`Vec2`/malformed/non-finite `Const` and `data`/`vec2_axes`/`look_at`/`follow` reject before render; the `keyframes` case must neither inspect nor claim its contents; and
- default-row source has no `onClick`, pointer/keyboard handler, `postMessage`, host-codec call, intent literal, or mock `state`/callback binding.

`PRIMARY_ORACLE`: decoder and AST/product-source guards prove the closed Position summary (`Const(Vec2)` values or tag-only `animated`), same-source grammar reuse, and inert default row. `REPO_LANES`: the two focused Node guard files, the existing Inspector product/ownership guard lane, `./scripts/check-docs.sh`, and `git diff --check`. `EXTERNAL_GATES`: human visual review remains deferred to the M3 final checklist; no WebView/native product-window, accessibility, or manual gate is executed or claimed by A.

## 4. Adoption preflight, negatives, and stop

```text
MECHANISM CLASS: read-only typed Position projection rendered through an existing product-owned row grammar
KNOWN IMPLEMENTATION SEARCH: InspectorCandidate installed/default branches; inspectorReadModelDecoder; existing Inspector post-promotion provenance/AST guards; InspectorHostRuntime snapshot/publish; U4b-0 prepare contract; ProductApp editor_playhead carrier
CANDIDATES: existing objectRow grammar in InspectorCandidate / existing decoder and guards / installed mock callbacks and literals / existing effect-param codec / new component or generic property framework
ADOPTION ROUTE: REUSE
REJECTED CANDIDATES: mock callbacks/literals/key count = not actual product state; effect-param codec = wrong command; new component/framework = duplicate owner; keyframe evaluation/playhead = B or later; fixed values or unsupported DocParam fallback = fabricated projection
THIN MOTOLII SEAM: closed Position summary (`Const(Vec2)` values or tag-only `Keyframes` presence) -> existing default branch's same-source row grammar
THIN MOTOLII RESIDUAL: exact fail-closed Const admission, tag-only Keyframes classification, and source/provenance guards
RETIREMENT: NONE; mock installed branch remains a mock consumer of its own state
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN
```

Negative cases: absent primary remains the existing null projection; missing transform/position omits the row; `Const(F64|Vec3|Color|AssetRef)`, non-finite Vec2, malformed Vec2, and `data`/`vec2_axes`/`look_at`/`follow` reject before render. `keyframes` is admitted only as tag-only `animated` presence; it must not yield a numeric value, a count, an ID, a time, interpolation, or a claim that its contents are valid. No partial row or fabricated value is allowed. A must stop if it requires current playhead evaluation, automation/key semantics, an IPC/message shape, a queue action, Rust Host changes, a new public/Document/schema/journal/plugin contract, or a second component/source.

Boundary B remains `WAIT_TARGET`: it alone may later establish the typed one-shot Inspector intent, private Host/queue consumer, `ProductApp::editor_playhead.current` input, and the already accepted `prepare_add_position_key(primary, current_playhead)` route. A does not change U4b-0, playhead, Easing, or AddPositionKey behavior.
