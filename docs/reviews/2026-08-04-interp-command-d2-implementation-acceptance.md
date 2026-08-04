# P04-C2 INTERP-COMMAND D2 implementation acceptance

- 日付: 2026-08-04
- 実装commit: `03667b7db574711c1eac201c41eb1735f5b40ab3`
- 状態: **DONE / ACCEPTED（D2 command sub-boundary のみ）**
- 正本: [INTERP-COMMAND D2 contract](2026-08-04-interp-command-d2-contract.md)

## 1. 受入結果と実diff

commit `03667b7d` は existing Position key の outgoing `Interp` だけを変更する
`Command::SetPositionKeyInterp` と `DocumentWriter::prepare_set_position_key_interp` を existing D2、
Undo、JournalEdit v2、WAL replayへ接続した。変更は次の5ファイルだけである。

- `crates/motolii-doc/src/command.rs`
- `crates/motolii-doc/src/lib.rs`
- `crates/motolii-doc/src/undo.rs`
- `crates/motolii-doc/src/journal/replay.rs`
- `crates/motolii-doc/tests/d2_command.rs`

same-value prepareはno-op、raw applyはold payloadをCASし、selected keyの`interp`以外のID、time、value、
他key、stable ID counterを変えない。inverseはold/newをswapし、same gestureかつ同じlayer/keyだけを
first-old/last-newへmergeする。terminal keyとone-key trackもD2 admissionでは許可し、strict-interiorの
left-key制約はfuture producerに残す。JournalEdit v2のtag/fieldとformat version、Document schema、依存、
公開面は契約で許可したdedicated commandとWriter wrapper以外に増やしていない。

## 2. validation と独立review

- `cargo fmt --check`: PASS
- `cargo test --locked -p motolii-doc --quiet`: PASS
- `cargo clippy --locked -p motolii-doc --all-targets -- -D warnings`: PASS
- `git diff --check`: PASS

fresh Opus 5 mediumのread-only独立reviewは
`ACCEPT / SCOPE PASS / MUTATION NONE / P0 NONE / P1 NONE / EVIDENCE_GAP NONE`を返した。
生streamは
`/private/tmp/motolii-external-logs/20260804-interp-command-impl-opus-medium-02`
に保存した。review findingから新しい施工権限を作らない。

## 3. 残る境界

`INTERP-COMMAND` nodeだけを`DONE / ACCEPTED`とする。通常製品のproducer、Host/React intent、popup、
inputは`WAIT_TARGET`のままで、親`P04-C2` / `U4b-1`は`TARGET_MISSING` / incompleteである。
InspectorのAdd Position Keyは別粒`CU-0A08ITI WAIT_TARGET`であり、本受入から接続しない。
native visual、focus、accessibilityを含むmanual gateはM3最終までdeferredのままである。
