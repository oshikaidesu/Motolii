# Frame status cost — 2026-09-09

Purpose: remove measured non-shader work from the current Flutter/native frame path without changing document values, glass, GPU publication ordering, or reflection quality.

## Evidence and scope

The running `dev.motolii.stage5` process was sampled on `light-in-form-observed-20260909.rrd`. In the short playback sample, 588/1569 native worker samples were in the two status requests and 789/1569 were in device polling. These are stack residence counts, not GPU timings or FPS. Raw captures and the original native library are in `/tmp/motolii-lag-20260909/` for this investigation.

Reference mechanisms inspected before implementation: `ProbeRuntime.render`'s existing status protocol; `EditorSession.liveLayers` and `overlayLayer`'s dynamic-value contract; `RecordCache`'s revision invalidation; `clipping_contract`'s parent, order, hidden, Undo, lock and saved-document rules. The installed wgpu Metal wait polls at 1ms intervals; this patch does not remove the completion wait.

Changes:

- `renderInfo` reads composition dimensions directly before allocating the output. Full status is still returned after rendering. The query does not mark a full snapshot as delivered, and reads current dimensions after composition edits.
- Clipping bases are computed together by order and cached for the document revision. Equal-order layers do not clip to one another; ties below select the highest layer ID. Preview edits use the existing uncached lookup so projected and persistent views cannot contaminate each other.
- Playback property rows omit keyframe curves and UI metadata that the existing live overlay never consumed. Playback also skips color swatches, effect layouts, blend previews, and static layer metadata that would otherwise be built and discarded. Dynamic values, text, bounds and key indicators remain present. Inspector data construction itself is not yet eliminated.

## Validation

- Clipping contract: 3 tests passed, including new equal-order and preview-isolation coverage.
- Native playback equivalence and dimension-query test: passed; dynamic property/effect values and key indicators match full status at three frames, and dimensions track composition edits.
- Native library and Swift/Flutter Debug host builds passed. Document unit suite: 43 passed. Existing playback progression contract: 1 passed. Stage 5 workspace check and diff whitespace check passed.
- Updated application was restarted and the saved comparison document opened. Actual window showed the scene at frame 0, playback at frame 8 and beyond, stopping, and seeking back to frame 0. This verifies operation, not a precise FPS or a complete absence of stutter.

Before-change repeated status queries on the saved 15-layer document (30 samples after warmup, Python FFI round trip including JSON decoding): paused median 12.39ms / p95 12.89ms; playing median 6.47ms / p95 7.22ms. This is a status microbenchmark, not end-to-end frame performance. Build activity and other apps make timings approximate.

Existing unrelated UI, text and document changes were present and are preserved. Snapshot pre-edit contents were copied before modifying that already-dirty file. No glass, reflection, output resolution or GPU synchronization behavior is changed.

After-change status microbenchmark on the same saved document: paused median 11.42ms / p95 12.43ms; playing median 4.06ms / p95 4.74ms. Playing status median decreased approximately 37%. In addition, the pre-render full status call is eliminated. Do not translate these figures into an FPS improvement percentage.

During concurrent workspace editing, a new visual-sample request used the original `model_reply` binding, so that binding was retained. Two accompanying compile errors were repaired narrowly: gradient stop editing shadowed the green-channel `g` with a gradient object, and a new `fill` projection had been inserted into `authored_signature` instead of the status layer builder. Their intended behavior is preserved. The source tree continued to receive unrelated changes during verification; this report only claims the checks and runtime observations above.

Remaining: Inspector data construction itself, the GPU completion wait and playback cadence, and reflection cache miss frequency. These require separate measurement and/or a completion-ordering design; this patch does not claim all perceived lag is eliminated.
