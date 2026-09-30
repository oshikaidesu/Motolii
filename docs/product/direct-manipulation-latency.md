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

## Invalidation and Camera pass (2026-09-30, third pass)
- Browser (`BrowserSession`), the Colors instrument, the Inspector seat and the Camera head listened to `layers` / `documentRevision`, so every preview tick rebuilt them. Each now compares what it draws (`colorReading`, `InspectorSession.shape`, the camera layer's name). `NativeVisualSample` already keys its request by value: it does not re-ask when a parent rebuilds. Pinned by `test/browser_invalidation_test.dart`. Still on `layers`: Stage chrome, Timeline, Ease, Relations (each shows layer values), Inspector stores (they are the rows being edited).
- Camera store: `absorb()` used to go through `set('camera.target')`, so a Target layer missing from the store's stale list was written back as None; it now only reads. A locked Target point shows the host's resolved centre in the real comp. Pinned by `test/camera_store_test.dart`.
- First move: the Dart half (first `previewProperties`, then `render`, nothing between) is pinned by `test/first_move_contract_test.dart`. What is not shown is the Stage pixel: any native op outside the allow-list at `port.rs` (`pause`, `select`, `preferences`, `stageView`...) landing between preview 1 and 2 cancels the native preview while the Inspector still holds the number. Measure in the real window (log op + `preview.is_some()`, and User-texture counters) before changing native.
- Open, not changed: Framing is applied by the resolver and the Stage camera gizmo but not by the production render camera (`camera_pose_parity.rs` pins it as known divergence); the product UI Camera instrument has no Framing row, so Distance cannot be shown as overridden. Face eye pitch sign vs Stage orbit (`camera_face.dart` vs `touch.dart`) needs a look in the window. Stage orbit 0.3 deg/px vs face 0.8 deg/px is unchanged.

## One path for "the newest wins, the ending keeps its place" (2026-09-30, fourth pass)
`EditorSession.commandDirect(op, args, key)` is now the only latest-wins primitive on the product UI path, and `command()` joins the same queue while previews are queued (before, it waited for the queue to drain, and updates queued meanwhile ran ahead of it: a gesture's `begin` could arrive after its own updates). Hand-written copies removed: `StageSession` gesture update, observer orbit and 3D hover; `DepthController`; the Ease adapter; `TimelineSession` seek and `previewTimings`; `ColorEdit`. A failing queued item no longer stalls the rest. Kept: `EditorPreviewQueue` and the loops in the Classic shell panels (`panels/*`, `timeline_core/frame.dart`, `grip.dart`), which the live shell does not use; `BlendController` (its wish also dedupes against what is already previewing and cancels on `null`).

Work no longer done: a seek to the frame already asked for; a `previewTimings` with the same changes as the last; per pointer event hull + visible-list rebuilds on the Stage (once per document state now); Browser / Colors / Inspector seat / Camera head / Relations / Ease rebuilds on previews that change nothing they draw.
Gained: the playhead follows the press itself (a `Listener`, not a tap + drag recogniser pair); Stage trackpad pan and pinch; a held arrow key adds to where the playhead was last asked to go, not to the last reply.

## Which requests end a live native preview (`preview_survival.rs`)
Ends it: anything that edits the document, moves the time (`seek`, `play`, a `pause` while playing), changes the selection, replaces the document (`new`, `undo`, `redo`, open), and `stageView` while a Stage *drag* is live (its geometry moves under the hand).
Leaves it: `status`, `tick`, `reloadEffects`, `stageWindow`, `renderInfo`, `visualSample`, `fontFacts`, hover, `preferences`, `pause` with nothing playing, `stageView` with no Stage drag.
So the hypothesis "a viewer-only request between preview 1 and 2 drops the preview" was true for `stageView`, `preferences` and an idle `pause` and is fixed for those. Whether the first-move symptom in the real window is one of them, or a texture republish, is measured only there: log native `op` + `preview.is_some()` around a first drag, and the User-texture counters.

## Not done, on purpose
- Timeline (`timelineSession`) still relanes on every `layers` tick: its rows hold layer maps that bars and lanes read, so a value-blind derived list needs the field list of every bar first.
- Camera view is redrawn for Stage-only changes (`renderInfo` always lists Camera; `_shownViews` is never read): a native + Swift change.
- A second Stage view shares one native `stage_window`; playback + a live preview render from two producers; native rebuilds the whole `layers` body per preview tick. All ownership-level.

## Fourth pass: what does not change with the hand, and what grows with the document
**Inspector, no whole-document JSON.** The Transform store used `Object.hash(jsonEncode(c.liveLayers()), ...)` as its "did the document change" test, once per notification (two per render: the slice and `rendered`). The session already replaces `layers` only when a reply's value differs (`EditorSession.take`), so the test was a string-built copy of a question the session answers. Now the Transform store compares what it read by value (no string, stops at the first difference), and the effect / layout / camera stores each compare *their own reading* (their rows, locked / frozen, capabilities, and only for a layer-picking row the other layers' names) and skip `_load` + notify when it is unchanged; before, every effect card reloaded and notified on every render of a Position scrub. On the 4-layer test document this is not visible in the probe (the numbers did not move); it removes per-render allocation that grows with the document.

**Stage drag / pan vs Inspector (about 8 ms).** Not extra Flutter work: Stage drag wakes the same three subscribers as the Inspector. The difference is in the render: `coherent = stage_drag.is_some() || !same_window` makes native wait for the GPU (a grab, and every pan step which moves the window, must show picture and window from one frame), and the Inspector path does not wait there. Caveat on the comparison: the Swift channel `renderInto` always calls `finish` before replying (it publishes the buffer itself, without the frame-ready callback), so for both paths the reply comes after the GPU is done; the gap is what native does around that for a window that moves. Changing it is the coherence design, not a missing simplification.

**Timeline scrub.** `seek` draws the head from a notifier at once, sends one keyed direct op, and wakes the Timeline slice 3 times in 40 steps; the top bar and the Ease host dedupe their own `frame` listeners. Nothing Timeline-side is synchronous with the picture.

**Subscribers left.** A Position / effect / camera preview, a Stage drag and a playhead scrub wake Stage, the Inspector session and the Depth desk (its `depthLayout` moves with position) per step, and nothing else.

**Dead / duplicate paths.** One ordered queue (`command` joins it while it runs, `commandDirect` replaces a same-key waiting item), no old drain chain, `commandNow` removed. Still alive, so left: `EditorPreviewQueue` (Classic fields, the rich-text editor, the Classic Ease desk) and the hand-written loops in `timeline_core/frame.dart`, `grip.dart` and `panels/depth_desk.dart` (the Classic / new shells are kept as capability sources and are built by `MOTOLII_SHELL`); none of them is on the live path.

## Growing with the document (measured, release native; `LATENCY_LAYERS=90` in `latency_test.dart`)
4 layers (layers JSON 32 KB): a preview command is about 17 ms. 94 layers (735 KB): about 84 ms, Stage drag 105 ms, Inspector scrub 92 ms, playhead 92 ms; Flutter build only 6.4 ms.
Inside the 94-layer render op (~69 ms): native CPU render 11.7 ms; a changed-snapshot status 17.6 ms (build 15.3 + serialize 2.4; payload 790 KB; inside the build: layer rows + geometry 3.4, main status json 2.8, scene + layer keys 1.4, depth layout 0.4; the other ~7 ms is outside `build_status`: a deep clone of the cached body for each reply and of the references, plus `to_string`); the remainder is the wait for the GPU (`finish`, needed because the channel path publishes the buffer itself) and ~7 ms of Swift / channel / Dart decoding. The wire is not the main cost.

## Next, if architecture is on the table
1. Publish through the frame-ready callback (as playback does) instead of `finish` before replying: takes the GPU wait off the reply, needs the texture / window coherence rule restated (renderer / ownership).
2. `status_response`: serialize the cached body without cloning it per reply, and do not rebuild or re-clone the static references: about 4-5 ms at 94 layers, native only.
3. A layer-row delta on the wire (only changed rows, unchanged rows as a stub the receiver expands): removes the O(document) payload; touches native, Swift (which broadcasts statuses to other windows) and Dart.
4. Skip `depthLayout` and the other desk-only keys unless a desk that reads them is open (needs a way to tell native).
