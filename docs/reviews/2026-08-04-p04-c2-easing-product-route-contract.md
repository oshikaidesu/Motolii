# P04-C2 Easing product producer / popup adoption contract

状態: **AMENDED / DIAGNOSTIC-CORRECTION DO / POPUP-TERMINAL CONTRACT_CLOSED / TERMINAL_VISUAL WAIT_TARGET / EXTERNAL_GATE_PENDING**

日付: 2026-08-04

## 1. 閉じる outcome

利用者が許可した通常製品routeは、product-owned React Easing trigger の click から native popup を
開き、選んだ区間の outgoing Position interpolation を既存 D2 command へ一回だけ送ることである。
これは既存 Stage transport の read-only `activeInterval` snapshot を input bridge へ変更しない別契約である。

```text
React Easing trigger click { anchor, layout_epoch }
  -> surface-local strict inbound route
  -> ProductApp / Host: current Document + primary + editor_playhead.current から再導出
  -> native popup session / admission
  -> one basic preset or validated custom Bezier release
  -> DocumentEditQueue Position-only action { LayerId, KeyframeId, Interp }
  -> SetPositionKeyInterp D2 prepare/CAS -> 1 journal command / 1 Undo / publish
```

`Document` が key、`Interp`、revision と Undo の唯一の正本である。React は anchor と
`layout_epoch`だけを送る presentation producer、ProductApp / Host は private identity 再導出・
session・stale admission、native popup は transient curve UI、DocumentEditQueue は single-writer
request、D2 は durable admission を所有する。popup token、projection generation、layout epoch、
screen coordinate、focus、drag preview は transient であり、queue payload、Document、journal、Undo、
User settings へ保存しない。

## 2. exact semantic / stale rule

1. React は direct product-owned `EasingTriggerCandidate` を reuse し、click 時に logical anchor と
   current `layout_epoch`だけを surface-local strict codec へ送る。Layer/key IDs、times、`Interp`、
   Document/revision は React と read-only Stage snapshot を越えない。
2. Host は受信時に current `Document`、primary、`editor_playhead.current`で既存 strict-interior
   Position interval を再導出する。interval 不在なら popup/session/queue write は 0。
3. opening session は private に layer、left/right key IDs/times、left `Interp`、projection generation、
   layout epoch と transient token を capture する。既存 G0-9 の placement、session、cancel patternを
   使用する。second device は禁止するが、product shared context の採択は実装で現行 owner を証明してから行う。
4. preset selection 又は custom Bezier release の直前に Host は current generation/layout epoch と
   strict-interior interval を再検証する。token closed、duplicate、cancel、generation/layout mismatch、
   key/time/left identity mismatch は queue enqueue 0 / Document write 0 で閉じる。
5. accepted terminal action は `DocumentEditQueue` のこの boundary で追加する Position-only action に
   `(LayerId, KeyframeId, Interp)`だけを一回 enqueue する。既存 D2 `SetPositionKeyInterp` prepare が old
   value を read し、command CAS が durable guard になる。一 basic preset 又は custom release は、**値が
   変わる時だけ** 1 queue action = 1 journal command = 1 Undo/publish、drag 中は 0 write である。same-value
   accepted terminal は no-op とし、queue action / command / journal / Undo / publish はすべて 0 で閉じる。

basic preset provenance is the React source authority
[`docs/mocks-ui/src/candidates/EasingGraphCandidate.jsx:16-41`](../mocks-ui/src/candidates/EasingGraphCandidate.jsx):
`Linear -> Interp::Linear`; `Smooth -> Bezier { .4, .0, .2, 1.0 }`; `Ease In -> Bezier { .42, .0,
1.0, 1.0 }`; `Ease Out -> Bezier { .0, .0, .58, 1.0 }`. Custom is `Interp::Bezier`, admitted by the
existing [`validate_interp`](../../crates/motolii-doc/src/doc_keyframe.rs) boundary. `Hold` is not in
that product source and is not part of this boundary. Bounce, Elastic and other advanced visual cards
emit no intent and are disabled until their semantic authority is separately closed; their presence in
an oracle does not authorize a durable mapping.

## 3. known-implementation adoption preflight

```text
MECHANISM CLASS: interval easing popup terminal admission over an existing D2 interpolation command
KNOWN IMPLEMENTATION SEARCH: P04-C2 ACTIVE-INTERVAL and INTERP-COMMAND contracts/acceptance;
  native Easing popup G0-9 acceptance; product-owned EasingTriggerCandidate and React source-authority
  EasingGraphCandidate preset values; Inspector surface-local strict codec/inbox pattern; existing ProductApp primary/playhead
  and DocumentEditQueue enum/action handling location
CANDIDATES: direct React trigger reuse; strict-interior PositionActiveInterval recomputation;
  G0-9 popup placement/session/cancel pattern; existing ProductApp GPU owner; existing queue enum/action
  handling location and D2 command
ADOPTION ROUTE: REUSE identities, D2 and trigger; add one Position-only action at the existing queue
  enum/action handling location; PATTERN for surface-local inbound,
  popup session/admission and exactly-once cancel/stale handling
REJECTED CANDIDATES: Stage activeInterval input bridge; SpikePresetStore; second wgpu device;
  test counters; hardcoded interval identity; generic popup/channel framework; new dependency;
  advanced interpolation semantics; User settings preset persistence
THIN MOTOLII SEAM: React anchor/layout intent is re-derived and admitted by the private Host session, then reaches the existing queue enum/action handling location.
THIN MOTOLII RESIDUAL: Position-only interval admission, stale rejection and product-specific oracle
RETIREMENT: static disabled trigger behavior once the product route is accepted; spike-only state,
  counters and persistence doubles remain non-product evidence
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN
```

The React source asset is not copied or reduced. The existing Stage transport `{snapshot, subscribe,
publish}` output bridge remains byte-for-byte the output-only bridge: no snapshot-object field addition
and no `postMessage` addition to that bridge are allowed. The inbound is a separate, surface-local Stage
transport sender/inbox contract, not a generalized snapshot/channel/Host framework. The G0-9 spike is a
pattern/oracle, not a store, second device, or product window implementation to import wholesale.

## 4. next implementation boundary and oracle

`P04-C2-EASING` is the next single product implementation boundary. Its exact source allowlist is
`ui/motolii-web/src/candidates/EasingTriggerCandidate.jsx`,
`ui/motolii-web/src/candidates/StageChromeCandidate.jsx`, the **new planned**
`ui/motolii-web/src/host/stage-easing-intent-codec.js`,
`ui/motolii-web/src/host/stage-transport-main.jsx`, and `ui/motolii-web/vite.host.config.js`;
`crates/motolii-ui/src/stage_chrome_host_runtime.rs`,
`crates/motolii-ui/src/product_runtime.rs`, and `crates/motolii-ui/src/document_edit_runtime.rs`; plus
focused tests in `ui/motolii-web/guard-tests/stage-easing-intent-codec.test.mjs`,
`crates/motolii-ui/src/stage_chrome_host_runtime.rs`, `crates/motolii-ui/src/product_runtime.rs`, and
`crates/motolii-ui/src/document_edit_runtime.rs`. Generated output is limited to the affected
`ui/motolii-web/generated-host/**` closure, its `asset-manifest.json`, and the corresponding
`crates/motolii-ui/src/browser_host_runtime.rs` `include_bytes!` / route-filename replacements;
content-hashed asset filenames are generated rather than fixed. No unrelated generated asset may
change. It may not alter `motolii-doc`, public APIs,
serde/journal schema, plugin contracts, User settings, dependencies, or Inspector/Add Position Key.

`PRIMARY_ORACLE`: the separate strict codec accepts only anchor/layout data; `Hold` input and every
advanced preset reject with intent/action/command 0; a current strict-interior
Position interval can open one session; every stale/cancel/duplicate/no-interval path performs zero
enqueue and zero Document write; a same-value accepted terminal also performs zero queue action,
command, journal, Undo, and publish; each value-changing accepted basic/custom terminal action reaches
exactly one `SetPositionKeyInterp` command, one journal command, one Undo/publish, and changes only the
left key's outgoing `Interp`. `REPO_LANES`: focused React/source-asset, Host/session/queue and D2 integration tests,
then relevant Rust/Node lanes, `git diff --check`, and `./scripts/check-docs.sh`. `EXTERNAL_GATES`:
native visual parity, real z-order/focus/dismiss, DPI/second monitor and accessibility remain for
the M3-final manual/real-device checklist; repository tests do not close them.

## 4.1 2026-07-22 native popup acceptance §5 reconciliation

This contract does not silently overturn [the 2026-07-22 native popup acceptance §5](2026-07-22-m3-native-easing-popup-acceptance.md#5-製品接続の停止線). Its stopping items are reconciled one by one:

- `U4a` interval/outgoing owner is discharged by `ACTIVE-INTERVAL` and `INTERP-COMMAND`; the latter is the accepted D2 owner.
- `U2h` primary projection is reused as the existing `ProductApp::primary`. No focus identity is invented; popup focus remains an external gate.
- The curve gesture's one-gesture / one-D2-command / one-Undo rule is this contract's value-changing terminal admission; same-value, cancel, stale, and duplicate terminals are zero-command paths.
- React trigger promotion/oracle is discharged through R2B, the Stage trigger acceptance, and the G0-9 pattern; no second React popup state is introduced.
- Platform z-order, focus, DPI/second-monitor, and accessibility acceptance is deliberately resequenced to the user-directed M3-final manual/real-device checklist and remains `EXTERNAL_GATE_PENDING`.

## 5. explicit non-goals / remaining waits

- Inspector Add Position Key stays separate as `CU-0A08ITI WAIT_TARGET`; no Inspector Position row,
  projection, or typed intent is inferred here.
- User preset save/delete/reorder/favorite persistence remains owned by the Host User settings codec;
  that work is `WAIT_TARGET` until its real codec is selected. Basic editing does not wait on it.
- Copy/paste, advanced Bounce/Elastic/Cyclic/Random/Steps/Elastic Steps semantics, a generic popup
  or input framework, new dependencies, public API/Document/schema changes, and a second GPU device
  are out of scope.
- Acceptance of this contract is not implementation, product E2E, or human/native visual acceptance.

## 6. 2026-08-04 terminal-adoption amendment

This amendment replaces the former single implementation order `P04-C2-EASING`. It closes exactly
two bounded tickets and does not authorize the partial React/IPC route currently outside the product
runtime.

### 6.1 `P04-C2-DIAGNOSTIC-CORRECTION` — `DO`

`crates/motolii-ui/src/diagnostic_projection.rs::command_kind_copy` is the already-existing exhaustive
consumer of `CommandKind`. `CommandKind::SetPositionKeyInterp` is accepted in D2, but that consumer
has no arm. The complete ticket is one arm returning exactly `"Set position key interpolation"` and
one focused assertion beside `clip_start_command_uses_the_existing_diagnostic_copy_route`. It adds no
product meaning, producer, popup, queue action, or public surface.

`PRIMARY_ORACLE`: the focused test proves the exact label for `CommandKind::SetPositionKeyInterp`; the
match remains exhaustive. `REPO_LANES`: the focused Rust test, relevant `motolii-ui` Rust lane, and
`git diff --check`. `EXTERNAL_GATES`: none. This is the only implementation `DO` emitted here.

### 6.2 `P04-C2-POPUP-TERMINAL` — contract closed; terminal visual route `WAIT_TARGET`

The future popup is product-local and private: `ProductApp` owns its transient popup state/module,
the one `EventLoop<ProductEvent>` constructed by `product_runtime::run` remains the only event loop,
and that same `ProductApp`-owned `Arc<GpuCtx>` remains the only product device/context. If a child
window is later admitted, `product_runtime_adapter.rs::window_event` must dispatch by its real
`WindowId` to that private owner; the primary-window path must not silently consume the child event.
No App/EventLoop/WebView, wgpu device, public popup abstraction, or generalized channel is authorized.

G0-9 is `PATTERN` only for placement, transient session/cancel rules, Bezier gesture/hit testing, and
their oracle. Do not adopt its `SpikePresetStore`, `UserPreset`, commit counters, `current_curve`,
revision state, or `PopupGfx`; none is a product owner. The partial React/IPC dead route is neither an
adoption candidate nor a fallback and must not be committed.

The known product rendering evidence is deliberately insufficient for a popup implementation:
`native_timeline_renderer::NativeTimelineRenderer::{new,prepare,composite}` and its private
`draw_text`/`TimelineFont` prove that the repository already uses Vello + fontique + harfrust for the
Timeline overlay, but their exact API consumes `NativeHostLayout`, `Document`, and
`TimelineProjection`; it exposes no popup-renderer or text-renderer route. `glyphon` is absent.
Copying or porting that private renderer, or introducing another renderer, would be bespoke framework
work and is forbidden. Therefore the terminal visual route remains `WAIT_TARGET`; this amendment does
not authorize popup code after the diagnostic correction.

```text
MECHANISM CLASS: native transient popup terminal over ProductApp's sole event loop and shared GPU
KNOWN IMPLEMENTATION SEARCH: ProductApp::run/ProductApp/GpuCtx; product_runtime_adapter::window_event;
  NativeTimelineRenderer and private draw_text; G0-9 popup spike; existing P04-C2 D2/queue contract
CANDIDATES: ProductApp private ownership; sole EventLoop<ProductEvent>; shared Arc<GpuCtx>;
  NativeTimelineRenderer's existing Vello/fontique/harfrust route; G0-9 placement/session/Bezier pattern
ADOPTION ROUTE: PATTERN only for G0-9 placement/session/Bezier/hit testing; no render adoption yet
REJECTED CANDIDATES: SpikePresetStore/UserPreset/counters/current_curve/revision/PopupGfx; glyphon;
  renderer port/copy; second App/EventLoop/WebView/device; generic popup/channel framework; partial React/IPC route
THIN MOTOLII SEAM: diagnostic CommandKind label only; popup visual seam has no selected existing target
THIN MOTOLII RESIDUAL: select a real product-local popup rendering target before terminal visual code
RETIREMENT: do not retain or promote the partial React/IPC route
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN
```

`P04-C2-POPUP-TERMINAL` is a docs contract ticket, not a code ticket. Its closure fixes the owner,
event/device exclusions, and adoption disposition above. It does not create a follow-on `DO`: after
`P04-C2-DIAGNOSTIC-CORRECTION`, the popup terminal visual implementation remains `WAIT_TARGET` until
an exact existing product rendering target and consumer are proven in a separate authority change.
