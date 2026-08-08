# P04-C2 diagnostic correction implementation acceptance

- 日付: 2026-08-04
- 実装commit: `58b84e22fe98fb6afe58016f9f548b6b776bfab4`
- 状態: **DONE / ACCEPTED（diagnostic projection sub-boundary のみ）**
- 正本: [P04-C2 Easing product route contract §6.1](2026-08-04-p04-c2-easing-product-route-contract.md#61-p04-c2-diagnostic-correction--do)

## 1. 受入結果と実diff

commit `58b84e22` は `crates/motolii-ui/src/diagnostic_projection.rs` だけを変更した。既存のexhaustive `command_kind_copy` に `CommandKind::SetPositionKeyInterp` の arm を追加し、正確に `"Set position key interpolation"` を返す focused assertion を既存のdiagnostic copy route test群へ追加している。

popup、React/IPC、queue action、product意味、public API、Document/schema/journal、dependencyは変更していない。

## 2. validation

- `cargo test --locked -p motolii-ui --lib diagnostic_projection`: PASS（5 passed）
- `cargo test --locked -p motolii-ui --lib diagnostic_projection --no-run`: PASS
- strict clippy、`cargo fmt --check`、`git diff --check`: PASS

`PRIMARY_ORACLE` は `SetPositionKeyInterp` のexact label assertionであり、match のexhaustive性を維持する。`EXTERNAL_GATES` はない。

## 3. 残る境界

このdiagnostic projection sub-boundaryだけを `DONE / ACCEPTED` とする。親 `P04-C2` / `U4b-1` は未完であり、popup terminal は `CONTRACT_CLOSED / TERMINAL_VISUAL WAIT_TARGET / EXTERNAL_GATE_PENDING` のままである。Inspector Add Position Key は別粒 `CU-0A08ITI WAIT_TARGET`、manual/native visual gate はM3 finalまで未実施である。
