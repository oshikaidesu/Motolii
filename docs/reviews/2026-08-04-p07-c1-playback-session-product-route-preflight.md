# P07-C1 mixed AudioProgram product PlaybackSession route preflight

状態: **TARGET_MISSING / PREFLIGHT ONLY / BUILD FORBIDDEN**

日付: 2026-08-04

## 1. outcome と停止線

P07-C1 が閉じるべき利用者成果は、Blender 等の通常 NLE と同じく、製品の再生制御が
**一つの audio-device clock** に従って mixed audio を再生し、同じ current time を Stage /
Timeline へ渡すことである。重い映像はその clock を遅らせず、最新時刻へ追いつくために
表示 frame を drop する。

この文書はその product route を実装しない。current main には次の4経路がいずれも
一つの通常製品routeとして閉じていないため、`P07-C1` は `TARGET_MISSING` のままである。
四つが同じ base で実在 source と focused negative oracle により閉じるまで、implementation
`DO`、child implementation ID、allowlist は作らない。

```text
actual product control source / typed intent
  -> ProductApp-owned playback session lifetime
  -> AudioProgram construction from current Document/project assets
  -> MixProducer -> device callback ring read only
  -> sole M2 Transport clock -> current-time handoff
  -> ProductApp render request + Timeline/Stage projection
```

`P02-C3` の `EditorPlayhead` は native ruler scrub の transient producer/carrier であり、
`PlaybackSession`、audio callback、continuous playback clock の代替ではない。scrub-only を
playback 接続として数えない。反対に、UI repaint／wall clock／vsync を product clock にせず、
固定 ZERO の `RenderRequest` を正常再生の代替にしない。

## 2. established design を継承するもの

- [D5 Transport prior art](2026-07-14-d5-transport-prior-art.md) と
  [decision index](../decision-index.md) は audio device を通常再生の sole clock と固定する。
  video は最新 perceptual time の frame を表示し、間に合わなければ drop する。自動
  varispeed、第二 clock、UI/repaint clock は棄却済みである。
- `crates/motolii-transport/src/lib.rs` の existing `Transport` は
  `PlaybackCounters::frames_supplied() - DeviceWaitLatency::wait_frames()` から
  `perceptual_time()`／`next_frame_plan()` を作る。D4-FU の resampler latency や ring
  fill量をこの時計から追加で引かない。
- `crates/motolii-audio/src/program.rs::AudioProgram::from_document` は Soundtrack と
  Clip audio component を deterministic に収集し、`mix_audio` を preview/export 同一入口とする。
  `crates/motolii-audio/src/producer.rs::MixProducer` は mix/decode を producer thread に置き、
  callback が ring read だけを行う既存の部品である。
- `crates/motolii-ui/src/product_runtime.rs` は private `ProductApp::editor_playhead` を native
  ruler/Stage/Timeline の current-time carrier として既に持つ。これは Document/journal/history
  に保存しない既存の read-side projection である。

```text
MECHANISM CLASS: NLE audio-clock playback, latest-time video presentation, typed product control
KNOWN IMPLEMENTATION SEARCH: D5 Transport prior art and D5 tests; AudioProgram/MixProducer AG-2;
  current PlaybackSession; P02-C3 ProductApp editor playhead; render-worker latest-generation path;
  current native Timeline/Stage projections
CANDIDATES: REUSE Transport, PlaybackCounters, DeviceWaitLatency, AudioProgram, MixProducer,
  existing ProductApp current-time projection and render-worker generation admission
ADOPTION ROUTE: REUSE/PATTERN only; established audio-clock NLE behavior is adopted through existing
  Motolii owners once the four exact product routes below exist
REJECTED CANDIDATES: UI/repaint/vsync or wall clock owner; automatic Transport varispeed; second
  clock; UI-owned mixer; fixed ZERO or ruler scrub as continuous playback substitute; new Space
  command/controller; new dependency; Document/journal/schema state
THIN MOTOLII SEAM: ProductApp lifetime and typed control bridge to existing audio/Transport owners,
  followed by current-time projection into existing render/Timeline/Stage consumers
THIN MOTOLII RESIDUAL: exact product session construction/lifetime, typed control admission,
  seek-generation handoff, and focused product oracle
RETIREMENT: any product UI timer/preview-only clock must be retired only after the same complete
  replacement route is accepted; none exists now
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN
```

## 3. current source facts and the four required closures

### A. AudioProgram / MixProducer construction — **MISSING / PREREQUISITE DO**

`AudioProgram::from_document` and `MixProducer::spawn_with_device_rate` exist, and
`crates/motolii-audio/tests/mix_program.rs` proves source ordering, mixed producer callback
nonblocking, and simulated 100 seeks. No product source constructs an `AudioProgram` from the
current `Document` plus a project root/cache owner. `motolii-ui` does not depend on
`motolii-audio` or `motolii-transport` in its current `Cargo.toml`.

Required closure fact: one existing product project/session owner must be named that can provide the
current immutable Document, project asset resolution root, and cache lifetime to `AudioProgram`; the
construction must not occur in the device callback or UI repaint. Missing/corrupt/unresolved assets
need an already-owned typed product failure path, not silent single-source fallback.

The source audit also found a narrower prerequisite: a zero-source `AudioProgram` currently gives
`MixProducer` an end frame of zero, so callback underrun does not advance the correct D5 clock.
[P07-C1A](2026-08-04-p07-c1a-video-only-program-supply-contract.md) reused the existing composition
duration as the shared producer supply floor and reached code/main `DONE / ACCEPTED` at commit
`d14010ad`. It does not close the missing product construction/root/cache owner described above.

### B. PlaybackSession constructor and lifetime — **MISSING / ADAPTER DO**

`crates/motolii-transport/src/playback.rs::PlaybackSession::{open_default,open_on_device}` is a
single-`Arc<PcmCache>` constructor and stores `_producer: AudioProducer`. It creates the output,
producer, counters, device-wait and `Transport` internally. Repository search finds no production
caller of `PlaybackSession`; its existing use is confined to its own crate/tests.

Required closure fact: a product-owned lifetime must own exactly one accepted session while playback
is active, stop/join it at product shutdown/reopen/failure, and name its error projection. The chosen
constructor must admit `AudioProgram`/`MixProducer` without keeping a parallel single-cache route.
No duplicate device output, callback-owned construction, hidden global, or new general controller is
allowed.

[P07-C1B](2026-08-04-p07-c1b-mixed-playback-session-contract.md) is the bounded adapter `DO`: it may
replace the existing session's single-cache producer with `AudioProgram` / `MixProducer`. It does not
select the missing ProductApp lifetime, construction caller or error projection.

### C. current-time producer / consumer handoff — **MISSING**

`ProductApp::editor_playhead` begins at `RationalTime::ZERO`, is changed by ruler scrub, and
`refresh_editor_playhead` republishes Stage plus a render request. The startup render template is
also initialized at ZERO. `ProductEvent::Wake` currently drains inspector/stage/browser work; it has
no `Transport::next_frame_plan()` consumer. Thus the existing ruler carrier proves neither a
continuous producer nor a handoff from the device counters to `RenderRequest`, native Timeline, and
Stage.

Required closure fact: one product owner must receive the current time from the sole `Transport`,
apply the existing latest-generation admission to the render request, and project the same accepted
time to Timeline/Stage. Playback and scrub precedence, seek generation invalidation, pause/idle,
and late producer/session shutdown must be fixed by existing-owner evidence before code dispatch.
The source must show that repaint frequency cannot advance the time.

### D. actual UI / Host control source and typed intent — **MISSING**

The current native timeline control surface has ruler scrub input, but the source audit found no
product `play`/`pause`/`seek` control, no Host/React transport command source, no typed transport
intent, and no `ProductApp` consumer. The map's historical wording “React transport/native scrub”
does not constitute a current source identity.

Required closure fact: select exactly one real product control source (or reduce to an already-real
source) and its direct typed intent consumer. It must retain the established keyboard/control design
without inventing a Space binding, controller, Host codec, React local playback state, or independent
UI timer. The selected source must prove visible normal entry, typed admission, stale/latest rule,
and failure/recovery routing.

## 4. admissible next disposition

`P07-C1` remains `TARGET_MISSING / PREFLIGHT ONLY`. P07-C1A is accepted and P07-C1B is the next bounded
adapter `DO`; neither invents a product owner. The next parent work after B is a
fresh read-only source audit that attempts to close the four listed facts one by one. If an existing
real control source is found but full playback remains unclosed, a future authority owner may choose
another explicitly bounded `REDUCE` slice. If any fact remains absent, that subfact stays
`WAIT_TARGET` and the other M3 lanes continue.

This is not an authority to implement a seek-only route, to change `PlaybackSession`, or to add a
dependency. It also does not unblock P07-C2 deadline policy or P07-C3 real-material measurement.

## 5. oracle and gates for a future closed boundary

`PRIMARY_ORACLE`: a product-route fixture must prove: one mixed `AudioProgram` reaches one device
callback whose work is ring read only; audio-device supplied frames are the only continuous clock;
current-time render/Timeline/Stage consumers agree; repaint/vsync storms do not change that clock;
latest seek wins and stale generation/session results publish zero; pause/stop/reopen tears down the
session cleanly; Soundtrack-only, two-source, Clip retime, and unresolved-source negatives retain
their typed disposition; preview/export sample semantics agree at the `AudioProgram` boundary.

`REPO_LANES`: focused audio mix/program tests, `motolii-transport` D5/render integration tests,
new product runtime/Host control tests only after the target exists, relevant Rust lint/format, and
`./scripts/check-docs.sh` plus `git diff --check`. Existing core green is not product-route proof.

`EXTERNAL_GATES`: M3-final manual native control/focus/affordance confirmation is deferred with the
other M3 human gates. Separately, P07-C3 retains its named real-material 10-minute audio-clock /
drift / drop / GPU-timestamp measurement; neither docs nor simulation close it.

## 6. non-goals

- Document, journal, Undo, schema, plugin contracts, and persistent playback state.
- New `Space` command, transport controller, dependency, Host generalization, or React playback store.
- Automatic varispeed, playback-rate policy, JKL/shuttle/audio scrub, waveform UI, DRS/pressure
  implementation, and P07-C2/P07-C3 work.
- Treating a fixture, simulated transport, single PCM cache, ruler scrub, or fixed ZERO render
  template as the normal mixed-audio product route.
