# P04-C2 ACTIVE-INTERVAL implementation-admissibility rejection

状態: **観察 / WAIT_CONSUMER**

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

`ACTIVE-INTERVAL` remains the existing node in the P04-C2 graph, but its local state is
`WAIT_CONSUMER`, not `IMPLEMENT` / `DO`. The parent `P04-C2` remains `TARGET_MISSING` because
the separate `INTERP-COMMAND` and product Easing route are absent.

Rejected workarounds: dummy/discarded read, `allow`/`expect(dead_code)`, `cfg(test)` or test-only
implementation, and an invented renderer, Host, React, popup, or Inspector consumer. None is a
product connection and each would falsify the compiler finding instead of resolving it.

## Resolution route and non-goals

`REMAP`: fresh preflight must first identify one real existing product consumer or Host intent
boundary that owns an observable use of the interval. This observation neither selects nor
authorizes that consumer. The separately `WAIT_TARGET` Inspector Position route is only a
candidate for that fresh preflight, not an implementation target here.

No code, Document/journal/history/Undo/queue/projection-generation mutation, public API, Host
codec, React projection/intent, popup, preset/settings, outgoing `Interp` command, or generalized
channel model is authorized by this observation.

Fable `ACCEPT_ISOLATED` was read-only consultation, never authority; the compiler oracle falsifies
its sufficiency for product implementation eligibility.
