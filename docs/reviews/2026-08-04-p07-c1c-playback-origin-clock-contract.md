# P07-C1C playback-origin audio clock contract

状態: **DO / CLOSED CONTRACT / IMPLEMENTATION PENDING**

日付: 2026-08-04

## 1. outcome と境界

`PlaybackSession` は `start_frame` から `MixProducer` を開始する一方、existing `Transport` は
`frames_supplied` を常にtimeline ZEROとして解釈する。非ZERO playheadから再生すると
`perceptual_time()` / `next_frame_plan()` がZEROへ巻き戻るため、product handoff前に同じclock内の
timeline originを閉じる。

本粒はaudio-device sole clockへsecond clockを追加しない。既存の供給済みframeとdevice waitの差へ、
session開始時のcanonical source frameをexact `RationalTime`へ変換して一度だけ加える。

## 2. 既知実装採択

```text
MECHANISM CLASS: media transport presentation timestamp origin
KNOWN IMPLEMENTATION SEARCH: existing PlaybackSession start_frame; Transport perceptual_frames/time;
  next_frame_plan; D5 sole audio-device clock; existing RationalTime/Fps exact frame conversion
CANDIDATES: A) Transport carries immutable timeline origin; B) pre-advance shared counters;
  C) ProductApp adds a second offset clock
ADOPTION ROUTE: REUSE A; one clock, origin + elapsed device-supplied frames
REJECTED CANDIDATES: B mutates device counters; C second clock; UI offset; wall/vsync clock;
  compatibility constructor retaining implicit ZERO
THIN MOTOLII SEAM: immutable RationalTime origin on existing Transport constructors and clock reads
THIN MOTOLII RESIDUAL: constructor call-site remap and focused nonZERO origin fixtures
RETIREMENT: implicit-ZERO Transport construction is replaced, not retained
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN beyond section 4
```

## 3. exact contract

1. `Transport` stores one immutable `timeline_origin: RationalTime`.
2. `Transport::new` and `Transport::new_with_gpu` accept that origin explicitly; no overload/default
   constructor is retained. Existing ZERO-based tests/simulators pass `RationalTime::ZERO` explicitly.
3. `PlaybackSession::open_on_device` converts its existing canonical `start_frame` exactly with
   `sample_frames_to_time(start_frame, CANONICAL_SAMPLE_RATE)` and passes the result to `Transport`;
   producer and Transport therefore share the same origin identity without mixing canonical and
   negotiated-device frame units.
4. `supplied_frames()` remains the raw device-supplied elapsed count and `perceptual_frames()` remains
   elapsed supplied minus device wait. `perceptual_time()` converts elapsed frames using the negotiated
   device sample rate and exact-adds the immutable origin; `next_frame_plan()` uses that absolute time.
5. At origin ZERO, all existing D5 behavior is unchanged. At canonical start frame `48_000`, zero
   elapsed supply reads exactly one second on both 48 kHz and 44.1 kHz negotiated devices; subsequent
   supply advances from that origin while device wait subtracts elapsed device frames only.
6. No ProductApp, React/Host, pause/seek policy, Document, journal, output callback, producer, DRS policy,
   dependency or UI change is included.

## 4. capsule and oracle

`ALLOWLIST`: `crates/motolii-transport/src/lib.rs`,
`crates/motolii-transport/src/playback.rs`, `crates/motolii-transport/src/simulate.rs`, and focused
`crates/motolii-transport/tests/*.rs` constructor/origin checks only.

`PRIMARY_ORACLE`: focused tests prove a one-second nonZERO canonical origin is present in
`perceptual_time()` and `next_frame_plan()` at both 48 kHz and 44.1 kHz device rates, device wait
subtracts only elapsed device supply, ZERO-origin behavior stays identical, and `PlaybackSession`
derives origin from the same `start_frame` passed to the producer. Fresh read-only review must find
P0/P1=0.

`REPO_LANES`: `cargo fmt --check`; `cargo test --locked -p motolii-transport`;
`cargo test --locked -p motolii-audio`; `git diff --check`; exact allowlist check.

`EXTERNAL_GATES`: P07-C3 real-device/real-material clock measurement remains pending. Product playback
control and M3-final human visual checks remain separate.

`NON-GOALS`: ProductApp lifetime/program construction/error projection; React play/pause/step;
editor_playhead publication; end-of-program policy; repeated seek/reopen; UI timecode formatting.
