# P04-C2 ACTIVE-INTERVAL Stage transport Easing trigger implementation acceptance

- 日付: 2026-08-04
- 実装commit: `68ab4b9dea6991c7b617847bcffb08e3e6f268ce`
- 状態: **DONE / ACCEPTED（ACTIVE-INTERVAL read-only consumer sub-boundary のみ） / EXTERNAL_GATE_PENDING**
- 正本: [Stage transport Easing trigger consumer contract](2026-08-04-stage-transport-easing-trigger-consumer-contract.md)

## 1. 受入結果と到達範囲

commit `68ab4b9d` は、private `ProductApp` の strict-interior Position active interval を通常製品の
Stage transport まで read-only に接続した。既存 product-owned `EasingTriggerCandidate` を直接再利用し、
既存 Stage transport の interval-easing slot へ `activeInterval: null` または selected layer display
name と literal `"Position"` のみを投影する。key/layer identity、times、outgoing `Interp` は Rust
内部に留め、Document の唯一正本性を変えない。

normal product route で active/null presentation と Stage transport Host publication は到達した。ただし
これは active interval の可視化だけであり、click、Host/React input、pressed mutation、popup、native
window、preset/settings、outgoing `Interp` command、Document/journal/history/Undo/queue/projection generation
write、public API、dependency、general Host framework、generalized channel を導入していない。

`P04-C2` 親と `INTERP-COMMAND` は未閉鎖の `TARGET_MISSING` のままである。`CU-0A08ITI` の normal
Inspector Add Position Key route も `WAIT_TARGET` のままであり、本受入から選定・実装認可されない。

## 2. 契約照合と生成物境界

`ProductApp` は既存 Position read rule の strict interior だけを読み、primary/document/playhead 不在、
endpoint、unsupported/non-`Vec2`、missing layer name を `null` として publish する。Stage transport
Host は private output snapshot を初期化と current document/primary/editor playhead の reconcile 時だけ
publish する。layout/bounds 変更だけ（active scrub なし）は publish しない。ただし active scrub 中の
layout 変更は既存 cancel lifecycle に従い、playhead が press-time へ変化した場合だけ、その playhead
reconciliation として publish する。React trigger は `pressed={false}` のままで handler を持たない。
play/step/timecode behavior は変更していない。

Vite の hashed Stage transport bundle、manifest、および既存 `browser_host_runtime.rs` の exact
`include_bytes!` filename/route は source change の機械的 delivery closure として更新された。これらの
generated-host outputs は source authority ではなく、build/check と diff review の対象である。無関係な
bundle asset、manifest entry、browser route は採用しない。

## 3. validation、review、既知red

- `cargo fmt --check`: PASS
- focused Stage transport tests: PASS, 3/3
- focused Position active-interval tests: PASS, 3/3
- focused Stage transport snapshot tests: PASS, 1/1
- focused generated asset/route tests: PASS, 1/1
- `cargo clippy --locked -p motolii-ui --all-targets -- -D warnings`: PASS
- React/Host Node guard lane: PASS, 15/15
- `npm --prefix ui/motolii-web run build:host`: PASS
- `npm --prefix ui/motolii-web run check:host`: PASS
- `git diff --check`: PASS

Opus medium initial review findings were fixed, then a fresh low final review returned `ACCEPT`.
An invalid tool-simulation session is not acceptance evidence and is excluded from this result.

The full `motolii-ui` lane was not rerun for this acceptance: its preceding `--lib` run was
194/194 PASS, while the known unrelated `cu110pt` source assertion was red. That assertion was
not changed or reclassified, and this acceptance does not count the full package lane as green.

## 4. external gate と次 edge

Native visual, focus, and accessibility confirmation remain `EXTERNAL_GATE_PENDING`; they are part
of the M3 final external checklist and are not replaced by the repository lanes above. This does
not complete an Easing popup or M3 itself.

The next possible edge is a separate `INTERP-COMMAND` owner/write-route preflight. It receives no
implementation authorization from this acceptance.
