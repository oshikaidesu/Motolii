# P07-C1A video-only AudioProgram supply contract

状態: **DO / CLOSED CONTRACT / IMPLEMENTATION PENDING**

日付: 2026-08-04

## 1. outcome と親停止線

`P07-C1A` は [P07-C1 product route preflight](2026-08-04-p07-c1-playback-session-product-route-preflight.md)
で見つかった局所前提だけを閉じる。音声sourceが0件の通常のvideo-only Documentでも、既存の
audio-device sole clockを進められるよう、`AudioProgram` が既存
`Document::composition.duration` まで正規mix無音をringへ供給できるようにする。

親 `P07-C1` の4経路、製品 `PlaybackSession`、UI control、current-time handoffは未閉鎖のままである。
本粒の完了を製品再生完成へ繰り上げない。

## 2. authority と既知実装採択

- `Document::composition.duration` は既存schemaの正準composition尺であり、`Document::validate`
  が正値を要求する。itemが0件でも存在し、clip終端もこの値を越えられない。
- `AudioProgram::from_document` は既に同じ `&Document` からprogram-level `master_gain` とsourceを
  構築する。新しいDocument owner、schema、journal fieldは要らない。
- 既存 `mix_audio` はsourceが0件または範囲を覆わない場合にも `Ok` のstereo zeroを返し、
  `MixReport::silence_frames`で正規無音として分類する。
- `RingConsumer::fill_or_silence` はringから読んだ値がzeroでも `frames_supplied`へ数え、callbackが
  補ったunderrun zeroだけを `silence_frames` / `underrun_events`へ数える。振幅による時計分岐はない。
- Exportは実音声sourceの有無で `AudioExportPlan::None` を選び、mixed export時のframe数も既に
  composition durationから求める。本粒はExportの採否predicateを変更しない。

```text
MECHANISM CLASS: video-only NLE playback on the existing audio-device clock
KNOWN IMPLEMENTATION SEARCH: Document composition duration/validation; AudioProgram/mix_audio;
  MixProducer shared end bound; RingConsumer supplied-vs-underrun provenance; export audio plan
CANDIDATES: A) reuse composition duration as a MixProducer supply floor; B) add UI/wall/vsync clock
ADOPTION ROUTE: REUSE A; retain the existing mix and ring paths without a video-only branch
REJECTED CANDIDATES: B; second/fallback clock; callback padding; amplitude classification;
  duration owned by ProductApp/PlaybackSession/MixProducer; new schema or dependency
THIN MOTOLII SEAM: copy the existing composition duration into AudioProgram and include it in the
  one shared producer end calculation
THIN MOTOLII RESIDUAL: one stored scalar, one getter, one end-bound change, focused tests
RETIREMENT: NONE; no parallel mechanism is introduced
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN beyond the allowlist in section 5
```

## 3. exact contract

1. `AudioProgram` stores the existing `Document::composition.duration` as a private
   `RationalTime` and exposes a read-only getter. `from_document` copies the value; it does not derive
   duration from items or audio sources.
2. The test-only direct-source constructor receives composition duration explicitly. It must not
   invent `max(source end)` as a substitute owner.
3. `producer.rs::program_end_frame` remains the only bound used by both identity and resample loops.
   It returns `max(existing source extent, composition duration in canonical frames)`. This is a
   one-sided supply floor that closes the zero/short-source freeze without changing the pre-existing
   behavior of a source extending beyond composition duration.
4. Frames in uncovered ranges come from existing `AudioProgram::mix_audio`, pass through
   `RingProducer`, and therefore advance `PlaybackCounters::frames_supplied`. Callback-fabricated
   underrun remains excluded from logical time.
5. `Transport`, `PlaybackCounters`, `DeviceWaitLatency`, callback code, Export selection, Document,
   journal, schema, and public product control remain unchanged. No special video-only producer path.

## 4. oracle

`PRIMARY_ORACLE`:

- a zero-source program with explicit nonzero composition duration is constructible and
  `mix_audio` returns exact zeros;
- its `MixProducer` supplies exactly the composition-duration frame count through the ring;
  `frames_supplied` advances for those zeros and callback underrun counters do not;
- a source shorter than composition duration supplies a zero tail through the same path;
- existing nonempty-source and resample tests remain green.

`REPO_LANES`: `cargo test --locked -p motolii-audio --test mix_program` plus relevant
`motolii-audio` tests, `cargo fmt --check`, `./scripts/check-docs.sh`, `git diff --check`.

`EXTERNAL_GATES`: **NONE** for this prerequisite. P07-C1 product route and M3-final human control
gate remain separate; P07-C3 retains the real-material drift/drop measurement.

## 5. implementation capsule

`OWNER`: `AudioProgram` for the copied composition duration; existing `program_end_frame` for the
shared producer bound.

`ALLOWLIST`:

- `crates/motolii-audio/src/program.rs`
- `crates/motolii-audio/src/producer.rs`
- `crates/motolii-audio/tests/mix_program.rs`

`NON-GOALS`: product/session/UI/Host wiring; pause/seek precedence; Document or persistence changes;
Export changes; new dependency; second clock; automatic varispeed; Timeline/Inspector visual work.

`COMPLETION`: focused tests prove the zero-source and short-source supply floor, diff stays inside the
allowlist, and a fresh read-only reviewer finds no P0/P1 or clock/export/schema expansion.
