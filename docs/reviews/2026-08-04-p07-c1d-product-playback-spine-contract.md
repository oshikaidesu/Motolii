# P07-C1D product playback spine contract

状態: **DO / CLOSED CONTRACT**

日付: 2026-08-04

## 1. outcome と背骨

通常製品のReact Stage transportに実在する`#play`を唯一の通常入口とし、既存`ProductApp`が
一つの`PlaybackSession` lifetimeを所有する。再生中はexisting audio-device `Transport`だけから
absolute current timeを読み、同じ`ProductApp::editor_playhead.current`へ採用して既存Stage render、
Stage transport publish、native Timeline描画へ渡す。pauseはsessionをdropし、そのcurrent timeを保持する。

本粒はstandard NLEのplay/pause spineを接続する一契約境界である。JKL、shuttle、loop、audio scrub、
waveform、複数session、別clock、汎用transport controllerは作らない。

## 2. 既知実装採択

```text
MECHANISM CLASS: NLE play/pause session lifetime and audio-clock presentation
KNOWN IMPLEMENTATION SEARCH: React StageTransportCandidate #play; StageChromeHostRuntime typed IPC/inbox;
  ProductApp event owner/editor_playhead/render worker/native Timeline; DocumentEditRuntime ProjectSession;
  AudioProgram::from_document; PlaybackSession; D5 Transport; Blender/NLE playhead pattern
CANDIDATES: A) ProductApp owns one PlaybackSession and projects its Transport time;
  B) React timer/store owns time; C) new transport controller; D) parallel preview clock
ADOPTION ROUTE: REUSE A and existing Stage Host one-shot intent pattern
REJECTED CANDIDATES: B/C/D; Space shortcut invention; UI/repaint/vsync/wall clock; parallel output;
  callback decode/mix; fixed ZERO; dynamic Timeline marker sizing
THIN MOTOLII SEAM: typed React toggle intent, ProductApp session option, project-root read, existing clock projection
THIN MOTOLII RESIDUAL: private lifecycle state, stale prepare rejection, focused product fixtures
RETIREMENT: current #play without handler and fixed-ZERO continuous playback substitute retire together
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN beyond sections 3-5
```

## 3. exact owner / route

```text
React StageTransportCandidate #play
  -> exact typed toggle-playback IPC
  -> StageChromeHostRuntime bounded one-shot inbox + ProductEvent::Wake
  -> ProductApp playback lifecycle owner
  -> AudioProgram::from_document(current_document, ProjectSession document parent, caches)
  -> one PlaybackSession::open_default(start at editor_playhead.current)
  -> PlaybackSession::transport_mut().next_frame_plan()
  -> ProductApp::editor_playhead.current
  -> existing Stage transport publish + render-worker latest generation + native Timeline redraw
```

1. `ProductApp` owns no more than one active session. React owns display/intent only and never advances time.
2. `DocumentEditRuntime` may expose only the existing `ProjectSession::document_path().parent()` as a private
   product-root read; it does not expose or clone the session.
3. Program construction/decode runs off the device callback and is generation-qualified. A pause or newer start
   makes an older preparation result stale; stale results open no device and publish no time.
4. Start time is the nonnegative current `editor_playhead` converted by the existing exact frame conversion at
   `CANONICAL_SAMPLE_RATE`; no `f64` frame math is added. The accepted session receives that same start frame.
5. While playing, winit wake/poll only schedules reads. It never contributes elapsed time. The accepted
   `FramePlan::timeline_time` replaces `editor_playhead.current`; the same value drives the existing render request,
   Stage transport snapshot, and native Timeline state. Existing render generation admission remains unchanged.
6. Toggle while playing pauses: read the latest Transport time once, drop the session, retain that time, publish once,
   and return event-loop control to the existing idle rule. Toggle while preparing cancels by generation.
7. At composition end, clamp to the existing composition duration, drop the session, publish once, and become idle.
8. Ruler primary scrub begins only after active/preparing playback is stopped. An accepted Document mutation retires
   active/preparing playback before a new snapshot is projected; selection-only publication need not rebuild audio.
9. Missing output device, decode/path, time conversion, Transport, and Host codec failures use typed existing error
   structure and the current product failure projection. No silent single-source fallback or fake playing state.
10. Stage snapshot adds only a playback-state projection needed to render the existing button as idle/preparing/playing.
    It does not add a React playback store, timer, or second source of truth.

## 4. fixed Timeline / Inspector non-goals

- Inspector remains the existing product-owned React asset and is not copied into egui.
- Native Timeline key markers remain fixed visual size. Marker x is time projection; clip/bar width is duration
  projection; zoom/viewport changes projection only. Shape/type/content must not change marker width.
- No Inspector redesign, Timeline density/semantic zoom work, dynamic marker width, waveform, or human visual gate is
  included. All M3 human visual checks remain deferred to the M3-final gate.

## 5. capsule and oracle

`ALLOWLIST`: product playback dependencies; `crates/motolii-ui/src/document_edit_runtime.rs`;
`product_runtime.rs`; `product_runtime_adapter.rs`; `stage_chrome_host_runtime.rs`;
`crates/motolii-ui/src/browser_host_runtime.rs`の既存Stage generated-asset embed/path表だけ; React Stage transport
candidate、entry/bridgeとfocused tests、通常のHost buildが出力するgenerated product assetだけ。
generated assetのhash名を旧名へ手動renameしたりminified output／manifestを手編集してembed表を回避しない。
`motolii-audio`, `motolii-transport`, Document, journal, render worker, native Timeline renderer/projection,
Inspector, Easing, and public API semantics are read-only.

`PRIMARY_ORACLE`: focused fixtures prove exact toggle codec and bounded inbox; stale preparation opens zero sessions;
only one session is active; ZERO and nonZERO starts use canonical exact frames; simulated 48 kHz and 44.1 kHz
Transport plans publish the same absolute time to render/Timeline carrier; repaint/wake count does not advance time;
pause/end/document-change retire the session and retain/clamp time; invalid/missing input is typed and creates no
parallel output. Fresh separate-family read-only review must find P0/P1=0.

`REPO_LANES`: focused UI Rust tests; Stage Host web guard/tests/build; `cargo fmt --check`;
`cargo test --locked -p motolii-ui`; `cargo test --locked -p motolii-transport`;
`cargo test --locked -p motolii-audio`; normal Host buildを二回実行した再生成安定性;
`./scripts/check-docs.sh`; `git diff --check`; exact allowlist.

`EXTERNAL_GATES`: real default-device playback, audible pause/end, focus/affordance, and visual motion remain
`EXTERNAL_GATE_PENDING` until the M3-final human pass; P07-C3 retains its separate real-material clock measurement.

`NON-GOALS`: seek command/UI, previous/next key, Space binding, looping, playback rate, JKL, waveform, audio scrub,
background cache framework, hot document audio rebuild, Export change, Document/journal/schema, public command/codec,
new dependency beyond existing workspace `motolii-audio`/`motolii-transport`, or M3 completion.
