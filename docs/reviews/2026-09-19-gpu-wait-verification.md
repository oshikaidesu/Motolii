# GPU completion off the UI thread — verification

Status: implementation in the working tree; final gates and the 20% target are not yet accepted. No commit.

## Changes in this continuation

- `motolii/ui/native/src/lib.rs`: asynchronous surface contention returns busy instead of sleeping; callback registration can be revoked before freeing user data; shutdown joins the polling thread before dropping the renderer. Window/ROI changes retain the synchronous same-frame path. A failed render no longer advances `last_window`.
- `motolii/crates/motolii-render/src/compositor/selection_bounds.rs`: each submitted selection readback owns its staging buffer. Completed buffers return to a bounded pool. Starting another view no longer waits for, discards, or relabels the previous view's map.
- `compositor/presentable.rs`, `compositor.rs`, `engine.rs`, `ui/native/src/snapshot.rs`: transfer readback ownership and preserve its layer IDs, view, window, and document image key until GPU completion. Only matching results update current geometry.
- `motolii/ui/macos/Runner/MainFlutterWindow.swift`: ordered completion publication on main rejects older queued callbacks after a newer synchronous frame. Completion also wakes Dart for the final pending render/selection update.
- `motolii/ui/lib/session/editor_session.dart`, `bridge/native_frames.dart`: retain the latest skipped request, retry it after completion, and do not mistake an unchanged playback tick for completion of a pending render. Record all Flutter frames and select the actual 12-second interval in the summarizer.
- `motolii/crates/motolii-render/src/engine/render.rs`, `engine/analysis.rs`: skip full-document video prefetch and alpha-shape resolution when no file layers exist. This was added after CPU sampling; performance still needs remeasurement with this change.

Reference: wgpu 29.0.4 `src/api/queue.rs:288` guarantees map callbacks for a submission finish before its `on_submitted_work_done` callback; it may run on a submit/poll thread. `src/api/device.rs:83` distinguishes nonblocking Poll from Wait. Existing media first-frame, alpha-shape, presentable/export, and selection-mask tests supply the behavioral oracles.

## Evidence location

Raw logs and native samples: `/tmp/motolii-gpu-finish.yXN6xE/`.
Summarizer: `docs/reviews/2026-09-19-evidence/gpu-wait-stats.py`.
The earlier handoff logs remain untouched in its listed scratchpad.

## Measurements so far (debug, light-in-form.rrd, 12 seconds)

These runs predate the no-file CPU shortcut. The user changed the pane layout/ROI between them, so they are observations, not the final controlled A/B comparison. One duplicate-process run (`async.log`) is excluded entirely; `async-clean.log` has one app process.

| Stage (ms: median / p90 / maximum) | Sync, sync.log | Async, async-clean.log |
|---|---|---|
| `_renderNow` rendered ticks | 9.54 / 49.07 / 72.46 | 10.86 / 17.98 / 35.28 |
| Camera render call | 3.69 / 26.86 / 45.23 | 5.90 / 7.54 / 14.56 |
| Stage render call | 3.76 / 18.77 / 22.90 | 2.87 / 6.92 / 15.72 |
| status | 1.67 / 2.88 / 5.55 | 1.78 / 2.83 / 3.54 |
| JSON decode | 0.05 / 0.10 / 0.14 | 0.08 / 0.12 / 0.17 |
| Flutter build, all measured frames | 10.81 / 47.62 / 81.54 | 8.23 / 20.16 / 39.19 |
| Flutter raster | 1.43 / 1.60 / 3.06 | 1.34 / 1.69 / 9.03 |
| Flutter total | 12.59 / 63.35 / 107.36 | 11.03 / 24.00 / 47.21 |
| Frames > 16.7 ms / > 33 ms | 131 / 129 | 182 / 3 |
| UI occupation / 12 seconds | 6182.00 ms / 51.52% | 4505.63 ms / 37.55% |
| Rendered ticks / all ticks / Flutter frames | 268 / 411 / 421 | 360 / 715 / 715 |

The fixture's authored content ends at frame 180 but transport continues past it; both rigs run 12 seconds from frame zero. The second half includes empty output. This does not prove 12 seconds of continuous heavy content.

`playback.sample` captured the remaining CPU work: the sampled `render_into_mode` branch had 83 samples in frame rendering and 39 in `warm_upcoming`. These are sample counts, not exact timings or GPU duration. Both prefetch and alpha-shape preparation resolved layers even with no file-source candidates.

## Gates obtained

```
flutter analyze: No issues found! (ran in 17.1s)
flutter test before: 00:21 +160 -10: Some tests failed.
flutter test after:  00:24 +160 -10: Some tests failed.
raw_dimension: clean
raw_color: 1 raw colours
material_import: 1 Material/Cupertino imports
```

Flutter failure sets were compared and are identical: desk_workspace, ease_interaction, module_contract, visual_selection, panel_placement, and five panel_layout_cost cases. Both lint violations are the pre-existing workspace_view.dart cases.

Native baseline first stopped in FSEvents registration. Saved pre-edit binary retry excluding the async test:

```
test result: FAILED. 90 passed; 4 failed; 10 ignored; 0 measured; 1 filtered out; finished in 64.59s
```

The excluded baseline async test passed alone:

```
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 104 filtered out; finished in 0.76s
```

The four baseline failures were every_effect_on_the_shelf_has_a_snapshot, every_example_builds_its_document, creating_particles_shows_the_particle_rows, and the_cage_follows_the_drawn_text_under_an_orbited_camera. Freeze passed in this run.

New native tests built, but execution stopped at shader watcher registration before entering the async completion assertions. Samples `native-after.sample` and `frames-after.sample` show `FileServer::watch → notify → FSEventStreamStart → f2d_register_rpc`; these are not passing test results. Final full native and renderer test outcomes remain open.

## Actual window observations

- Launched and inspected both Stage and Camera; playback changes appeared in both. At frame 348 the output was empty because the authored layers end at 180.
- Clicking the timeline at frame 40 and 149 restored the rendered artwork in both views. The attempted continuous ruler drag did not move the frame with this computer-use input; do not count that attempt as a passed continuous scrub.
- Dragged the Stage/Camera divider from x492 to x624 and back. Both views resized; no distortion was visible in the observed resulting frames. Automated Fit/In/Out/Actual/Fit also ran.
- Selected Text, dragged Position X from -485.00 to -439.40, then one Edit → Undo returned it to -485.00 and cleared dirty state. Cmd-Z and accessibility-index clicks did not activate Undo in this run; a coordinate click did. Esc/focus cancellation is covered by the unchanged Flutter interaction tests, not claimed as manual mid-drag verification.
- Existing red `No Material widget found` headers appeared before and after; workspace_view.dart was not modified here.

Example real resize probe:

```
PROBE room=stage-window verdict=matched lag=0 frames path=ffi-same-frame window=1522x952 roi=[-837.7870810888097, -523.616925680506, 3275.5741621776197, 2047.233851361012]
```

No nonzero lag was found in the collected probes. This is window/ROI evidence, not a proof that no transient tear can ever occur.

## Outstanding before completion

1. Final controlled sync/async pair after the CPU shortcut, unchanged document and ROI; verify <20% occupation rather than claiming it from the earlier run.
2. Finish native and renderer suites; run the strengthened busy-surface, callback lifetime, and independent selection-readback assertions.
3. Run the rebuilt zz_watch and compare its known frame with `watch-before/0000.png`; exercise real export_range_with_progress and inspect the encoded result.
4. Repeat final-window checks with the final dylib.

A separate Claude process was observed building this same tree with a queued command to terminate Flutter/Motolii and relaunch it. Its commands were not interrupted. Final-window operations were paused for coordination with the user; this also explains unexpected app replacement during testing. User-authorized discarded edits were not saved over light-in-form.rrd.
