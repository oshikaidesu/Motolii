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
