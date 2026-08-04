# P04-C2 Easing product producer / popup adoption contract

状態: **決定 / IMPLEMENT（P04-C2 Easing product route） / EXTERNAL_GATE_PENDING**

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
  -> DocumentEditQueue { LayerId, KeyframeId, Interp }
  -> existing SetPositionKeyInterp D2 prepare/CAS -> 1 journal command / 1 Undo / publish
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
   使用し、shared GPU context で native popup を開く。
4. preset selection 又は custom Bezier release の直前に Host は current generation/layout epoch と
   strict-interior interval を再検証する。token closed、duplicate、cancel、generation/layout mismatch、
   key/time/left identity mismatch は queue enqueue 0 / Document write 0 で閉じる。
5. accepted terminal action は `DocumentEditQueue` に `(LayerId, KeyframeId, Interp)`だけを一回 enqueue
   する。既存 D2 `SetPositionKeyInterp` prepare が old value を read し、command CAS が durable guard に
   なる。一 basic preset 又は custom release は 1 queue action = 1 journal command = 1 Undo/publish、drag
   中は 0 write である。

basic presets are exact: `Linear -> Interp::Linear`; `Smooth -> Bezier { .4, .0, .2, 1.0 }`;
`Ease In -> Bezier { .42, .0, 1.0, 1.0 }`; `Ease Out -> Bezier { .0, .0, .58, 1.0 }`;
custom is `Interp::Bezier` after the existing finite and `x1/x2 in [0, 1]` validation. `Hold` is
available only where the product-owned source actually exposes it. Bounce, Elastic and other
advanced visual cards emit no intent and are disabled until their semantic authority is separately
closed; their presence in an oracle does not authorize a durable mapping.

## 3. known-implementation adoption preflight

```text
MECHANISM CLASS: interval easing popup terminal admission over an existing D2 interpolation command
KNOWN IMPLEMENTATION SEARCH: P04-C2 ACTIVE-INTERVAL and INTERP-COMMAND contracts/acceptance;
  native Easing popup G0-9 acceptance; product-owned EasingTriggerCandidate; Inspector surface-local
  strict codec/publish pattern; existing ProductApp primary/playhead and DocumentEditQueue
CANDIDATES: direct React trigger reuse; strict-interior PositionActiveInterval recomputation;
  G0-9 popup placement/session/cancel pattern; shared wgpu context; existing queue and D2 command
ADOPTION ROUTE: REUSE identities, D2, queue and trigger; PATTERN for surface-local inbound,
  popup session/admission and exactly-once cancel/stale handling
REJECTED CANDIDATES: Stage activeInterval input bridge; SpikePresetStore; second wgpu device;
  test counters; hardcoded interval identity; generic popup/channel framework; new dependency;
  advanced interpolation semantics; User settings preset persistence
THIN MOTOLII SEAM: React anchor/layout intent -> private Host re-derivation/session -> existing queue
THIN MOTOLII RESIDUAL: Position-only interval admission, stale rejection and product-specific oracle
RETIREMENT: static disabled trigger behavior once the product route is accepted; spike-only state,
  counters and persistence doubles remain non-product evidence
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN
```

The React source asset is not copied or reduced. The existing Stage transport contract remains
output-only; this new inbound is surface-local and does not generalize its snapshot, channel model,
or Host runtime. The G0-9 spike is a pattern/oracle, not a store, second device, or product window
implementation to import wholesale.

## 4. next implementation boundary and oracle

`P04-C2-EASING` is the next single product implementation boundary. Its allowlist is limited to the
existing product Easing trigger closure, its Stage/surface-local Host runtime and ProductApp
coordinator, the product-local adoption of the G0-9 native popup/session pattern, the existing
`DocumentEditQueue` action handling location, and focused tests for the route. It may not alter
`motolii-doc`, public APIs, serde/journal schema, plugin contracts, User settings, dependencies, or
Inspector/Add Position Key.

`PRIMARY_ORACLE`: the strict codec accepts only anchor/layout data; a current strict-interior
Position interval can open one session; every stale/cancel/duplicate/no-interval path performs zero
enqueue and zero Document write; each accepted basic/custom terminal action reaches exactly one
existing `SetPositionKeyInterp` command, one Undo/publish, and changes only the left key's outgoing
`Interp`. `REPO_LANES`: focused React/source-asset, Host/session/queue and D2 integration tests,
then relevant Rust/Node lanes, `git diff --check`, and `./scripts/check-docs.sh`. `EXTERNAL_GATES`:
native visual parity, real z-order/focus/dismiss, DPI/second monitor and accessibility remain for
the M3-final manual/real-device checklist; repository tests do not close them.

## 5. explicit non-goals / remaining waits

- Inspector Add Position Key stays separate as `CU-0A08ITI WAIT_TARGET`; no Inspector Position row,
  projection, or typed intent is inferred here.
- User preset save/delete/reorder/favorite persistence stays `WAIT_TARGET` pending a real User
  settings codec; basic editing does not wait on it.
- Copy/paste, advanced Bounce/Elastic/Cyclic/Random/Steps/Elastic Steps semantics, a generic popup
  or input framework, new dependencies, public API/Document/schema changes, and a second GPU device
  are out of scope.
- Acceptance of this contract is not implementation, product E2E, or human/native visual acceptance.
