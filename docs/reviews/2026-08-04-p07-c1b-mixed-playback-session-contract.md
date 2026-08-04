# P07-C1B mixed PlaybackSession adapter contract

状態: **DO / CLOSED CONTRACT / IMPLEMENTATION PENDING**

日付: 2026-08-04

## 1. outcome と境界

`P07-C1B` は既存 `PlaybackSession` の入力とproducerを、single `PcmCache` / `AudioProducer` から
existing mixed `AudioProgram` / `MixProducer` へ置換する。P07-C1Aでzero-source programの供給floorが
成立したため、Soundtrack、Clip audio、複数source、video-onlyは同じsession adapterへ入れる。

ProductApp lifetime、Document/project-root/cacheからのprogram構築、React control、current-time handoffは
本粒へ含めず、親P07-C1は`TARGET_MISSING`のままとする。

## 2. 既知実装採択

```text
MECHANISM CLASS: mixed-program audio-device playback session adapter
KNOWN IMPLEMENTATION SEARCH: existing PlaybackSession output/counters/device-wait/Transport owner;
  existing AudioProgram/MixProducer; canonical_format; negotiate_output; P07-C1A zero-source supply
CANDIDATES: A) replace the single-cache producer in the existing session; B) add a parallel mixed session
ADOPTION ROUTE: REUSE A; retain constructor names, output negotiation, counters and Transport construction
REJECTED CANDIDATES: B; second device output; callback mixing; UI-owned mixer; new session/controller/dependency
THIN MOTOLII SEAM: Arc<AudioProgram> input, canonical stereo negotiation, MixProducer lifetime field
THIN MOTOLII RESIDUAL: one existing file and compile-focused contract checks
RETIREMENT: the single-PcmCache PlaybackSession route is replaced, not retained
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN beyond section 4
```

## 3. exact contract

1. `PlaybackSession::{open_default, open_on_device}` keep their names and other arguments but replace
   `Arc<PcmCache>` with `Arc<AudioProgram>`.
2. The existing one output stream, `PlaybackCounters`, `DeviceWaitLatency`, negotiated output and
   `Transport` construction remain the sole owners.
3. Device negotiation and ring shape use existing `canonical_format()` / canonical stereo because
   `AudioProgram::mix_audio` has that established output contract.
4. The session stores one `MixProducer`; it calls `MixProducer::spawn_with_device_rate` with the same
   program, ring, start frame, negotiated device sample rate and no new meter owner.
5. `PcmCache` and `AudioProducer` leave `playback.rs`; no compatibility overload or parallel route is
   kept. Drop continues to stop/join producer through existing field Drop.
6. No change to `AudioProgram`, `MixProducer`, callback, Transport, Document, Export, product UI or Host.

## 4. capsule and oracle

`ALLOWLIST`: `crates/motolii-transport/src/playback.rs` only.

`PRIMARY_ORACLE`: the crate compiles and tests with constructors accepting `Arc<AudioProgram>`;
`playback.rs` contains `MixProducer` and canonical format negotiation, and contains no `PcmCache`,
`AudioProducer`, second output or callback-side mix. A fresh read-only reviewer must find P0/P1=0.

`REPO_LANES`: `cargo fmt --check`; `cargo test --locked -p motolii-transport`;
`cargo test --locked -p motolii-audio`; `git diff --check`; exact one-file allowlist check.

`EXTERNAL_GATES`: P07-C3 real-device/real-material audio-clock measurement remains pending and is not
closed by compilation. P07-C1 product-route and M3-final human control gates remain separate.

`NON-GOALS`: product dependency/lifetime/error projection; project root/cache ownership; typed play/pause
intent; pause/seek/reopen precedence; current-time projection; UI, Timeline or Inspector visuals.
