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
