# Direct manipulation: input must not wait for the previous picture (2026-09-30)

Measured in the real app (`integration_test/latency_test.dart`, debug Flutter; LAT lines) and read from the code. Not a renderer benchmark.

## What the path is
pointer -> UI preview -> `command` (serial chain `_serial`) -> native `request` (~0.5 ms round trip) -> `render` (native, on the platform thread, synchronously) -> reply -> next frame.
The native round trip is not the cost. A picture costs one native render plus the next frame, and native runs `request` and `render` one after another on one thread, so a new input can never be applied *during* a render; what can be controlled is that nothing queues behind it.

## Causes found and fixed
| Path | Cause | Before | After |
|---|---|---|---|
| Stage pan / zoom / wheel | the window native draws was asked for from the next build (`addPostFrameCallback`), a whole frame after the hand | 66 ms / 2 frames | 28 ms / 1 frame |
| Stage pan, long 125 Hz hand | one `stageWindow` command per input, each with a render, queued in the serial chain: the picture trailed the hand and caught up long after it stopped | last picture 250 ms after the hand stopped (13 stale renders) | 22 ms (2 renders) |
| live Inspector scrub (transform, effect, layout, camera rows) | `previewProperties` per tick straight into the serial chain (no latest-wins queue, unlike Classic's fields) | last picture 338 ms after the hand stopped | 27 ms |
| Inspector toggle switches | 120 ms knob slide on a press | 120 ms | 0 |

`EditorSession.commandDirect(op, args, key)`: continuous previews replace the waiting preview of the same gesture (newest wins); a commit, a cancel or any other command keeps its place after the previews before it (`command()` waits for the direct queue). No render is skipped that a later state has not superseded.

## Kept on purpose
- Resize / viewport: the viewport is only known during layout, so `stageWindow` after a resize still goes after the build (a command may not be sent inside a build).
- 300 ms settle timers on wheel / trackpad steps: gesture completion (the preview itself is immediate).
- Post-frame focus / `ensureVisible` in reveal: navigation, not a value.
- Stage drag, playhead scrub, timing preview, colour, text, ease: already latest-wins.
- `EditorPreviewQueue` awaiting the previous command (which includes its render): native applies one call at a time on one thread, so cutting it would not let the newest value in earlier.

## Still renderer / platform
The render itself (a few ms release, ~15-20 ms debug), `frames.finish()` waiting for the GPU while a Stage grab or window change keeps picture and window coherent, and native running on the platform thread, which is also the thread that delivers pointer events. Moving `render` off that thread is the next structural step and is a design change, not done here.

## Threading, read from the code (2026-09-30, second pass)
- Every runtime call (`request`, `render`, `renderInto`) already runs on a dedicated serial queue (`motolii.render`). Render is **not** on the platform thread: the channel handlers call it with `renderQueue.sync`, so the main thread (which Flutter 3.47 merges with the UI thread) *waits* for it. Playback already has an asynchronous path (`enqueuePlayback`), which drops a frame instead of queueing.
- Document mutation and render both take `&mut EditorRuntime`, so they cannot overlap; a render worker would need a snapshot of the document view, which is an ownership change (not done).
- What was tried: making the `request` and `render` channel handlers asynchronous (work stays on the render queue, in order; only the reply, broadcast and texture publish are on the main thread). It is safe (same order, same owner) but **slower** in the same-session A/B: Stage drag median 43 -> 63-66 ms (p95 60 -> 105-130), playhead 33 -> 46 ms, Inspector p95 51 -> 68 ms, because each reply needs a main-thread turn (three per drag update) and under a debug Flutter frame each turn can wait for the frame in progress. Reverted. If the hops are collapsed to one, or measured under a profile-mode Flutter, it may pay; it was not shown to.

## Inspector first move (checked, not reproduced)
`integration_test/first_preview_test.dart` presses each Inspector number, makes exactly one non-zero move (24 px and 6 px) and looks at what the host has: a `previewProperties` is sent by that first move and the host has a live preview open, for every transform row, the effect parameter row and the camera rows (camera centre / target rows are locked while the camera aims at a layer, by design). Native never cancels a preview on `stageWindow` / `renderInfo` (answered before the edit path). The test stays as a regression guard.

## Latest-wins loops in the code
Hand-written copies of "one in flight, the newest waits": `EditorPreviewQueue` (Classic fields, colour, text, ease handles), `StageSession._update`, hover and orbit, the timeline's `pumpSeek` / `_pumpPreview` / `pumpPreview`, the Ease desk's `_flight`, the Depth desk's `_drain`. They differ only in what they send. `EditorSession.commandDirect` is the session-level version that also keeps a commit or cancel *after* the previews before it. Folding the copies onto `EditorPreviewQueue` is mechanical and was not done: it changes no latency.

## What is left (and why it was not done)
- Flutter's own frame while scrubbing: build ~10 ms median in a debug Dart build (about 110 widgets rebuilt per frame). About half is the Browser panel: `BrowserSession` listens to the `layers` / `documentRevision` slice, so every preview of a transient document rebuilds the Create shelf tiles. Cutting it means deciding which document facts the Browser shelves really read (selection kinds, the selected layer's colour, fonts in use); a wrong list leaves a stale shelf, so it is a semantics review, not a mechanical change.
- Render off the platform thread, or one-hop async replies: an ownership / threading change; the simple version was measured slower (above).
- Renderer itself: a still render is one native call of ~5 ms (release) to ~15-20 ms (debug), plus the wait for the next frame.
- Keyboard nudge sends one command per key repeat (each with a render); nudges add up, so they cannot be replaced by the newest one, and at OS key-repeat rates the chain does not build a backlog.

## Panels that woke on every preview (third pass)
Counting slice notifications (`LatencyProbe.count`) during a 60-step Inspector scrub showed **eight** slices waking on every step: browserSurface, liveColors, stage, rightSeat, inspectorSession, timelineSession, liveEase, depth. Most read `layers` or `documentRevision` (which change on every preview) but show no value.

What each really reads, and what it listens to now:
- **Browser** (all shelves): palette, font families, assets, environments, the selection; through getters, the fonts the layers use, the active layer's kind and font, and the colour being edited. Now those keys plus a `derived` of exactly those getters. `ThingFace` / `_Tile` / `FutureBuilder` / `NativeVisualSample` read only `BrowserSession`; a sample or a picture is cached by its request key (`_SampleShelf`, `_pictures[id] ??=`), so a parent rebuild never asked native again (checked in the code; nothing to fix there).
- **Colors**: only the colour target (`colorTarget(c)`, which reads a layer's colour property when the host names none) -> `derived: colorTarget`.
- **Timeline, right seat (which Inspector), Ease**: what the layers *are* (timing, flags, which properties have keys where), not values -> `EditorSession.layerShape()` (values, positions, bounds, key values stripped).
- **Media seat**: `assets` and `backgrounds` only (was: the whole document).
- Still woken per step, on purpose: Stage (draws the value), Inspector session (shows it), Depth desk (its `depthLayout` moves with position).

Measured with the same Inspector scrub (debug Dart): slices woken per step 8 -> 3; widgets rebuilt over the same frames 1751 -> 359; Flutter build 11.6 -> 4.3 ms median; pointer-to-visible median 34 -> 25 ms (p95 51 -> 41 ms). A Stage drag and a playhead scrub wake the same three per step. Full real-app suite (15 files) passes.
