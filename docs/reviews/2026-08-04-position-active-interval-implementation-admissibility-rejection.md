# P04-C2 ACTIVE-INTERVAL implementation-admissibility rejection

状態: **歴史観察 / REMAPPED**

日付: 2026-08-04

## Compiler oracle

Preserved implementation wave on base `34177e3b82d0112a275c85055e8c1db0b6edafcd` added only the
contract-shaped private `PositionActiveInterval` and `position_active_interval` derivation plus
focused tests in `crates/motolii-ui/src/product_runtime.rs`. Focused
`cargo test -p motolii-ui --lib product_runtime` passed **34/34**, including exact left/right
identity/time/outgoing `Interp`, negative cases, and serialized Document equality.

Production admission failed at `cargo clippy -p motolii-ui --all-targets -- -D warnings`:

```text
error: struct `PositionActiveInterval` is never constructed
   --> crates/motolii-ui/src/product_runtime.rs:209:8
    = note: `-D dead-code` implied by `-D warnings`

error: function `position_active_interval` is never used
   --> crates/motolii-ui/src/product_runtime.rs:2203:4
```

The test result is fixture evidence only; it does not establish a normal product route. The
compiler oracle proves that the current ProductApp has no admitted production consumer for this
read model.

## Disposition

This compiler result was a valid `WAIT_CONSUMER` observation at the preserved base; it is not a
current prohibition. The [Stage transport Easing trigger consumer contract](2026-08-04-stage-transport-easing-trigger-consumer-contract.md)
later selected an existing ordinary product mount, exact disabled slot, and product-owned trigger.
Consequently `ACTIVE-INTERVAL` is now `IMPLEMENT` only for that contract's private output
projection. The parent `P04-C2` remains `TARGET_MISSING` because the separate `INTERP-COMMAND`
and product Easing route are absent.

Rejected workarounds: dummy/discarded read, `allow`/`expect(dead_code)`, `cfg(test)` or test-only
implementation, and an invented renderer, Host, React, popup, or Inspector consumer. None is a
product connection and each would falsify the compiler finding instead of resolving it.

## Resolution route and non-goals

`REMAP` completed only for the private Stage transport output projection. The selected consumer
does not create Host intent: it uses `activeInterval: null | { objectName, channel: "Position" }`
and retains IDs/times/`Interp` in Rust. The separately `WAIT_TARGET` Inspector Position route is
not selected by this remap.

This historical observation does not authorize Document/journal/history/Undo/queue/projection-generation
mutation, public API, Host input codec, React intent, popup, preset/settings, outgoing `Interp`
command, or generalized channel model. The separate private Stage output projection is authorized
only by the consumer contract.

Fable `ACCEPT_ISOLATED` was read-only consultation, never authority; the compiler oracle falsifies
its sufficiency for product implementation eligibility.
