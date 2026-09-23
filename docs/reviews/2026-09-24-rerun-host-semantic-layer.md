# Motolii is the semantic layer on a Rerun host (architecture C)

Adopted by the user on 2026-09-23. The migration ran overnight into 2026-09-24. This document updates the 2026-08-20 reset ruling ([reset-to-one-axis](2026-08-20-reset-to-one-axis.md)) because its premise changed. Pulling `re_renderer` directly is still the only path. What changes is who owns GPU execution.

## The ruling

- **Motolii** owns only the meaning of editing, composition and effects, plus the recipes and data it hands to the host (Scene, Prepared Frame, cassettes). **GPU execution belongs to the host:** the frame boundary, pools, belts, pipelines, bind groups and readback. Motolii is the embedder that calls `begin_frame` / `before_submit`, as eframe does for `re_viewer`. It records into encoders whose buffers it hands to `RenderContext::queue_commands`, as `ViewBuilder::draw` does.
- **Production uses only public Rerun API, or fork patches that could go upstream.** The fork (`oshikaidesu/rerun`, branch `motolii/host-core`) is a staging area for upstream candidates. Every fork diff is classified as UPSTREAM_NOW / UPSTREAMABLE / MOTOLII_SEMANTICS / DELETE. Only UPSTREAMABLE stays. No Motolii product concept may remain in the fork: Glass, Plate, Matte, Feedback, Block rope, extrusion, `motolii_*`. **The oracle for any fork patch is a green Viewer build.**
- **A cassette is WGSL/Vism + parameters + generic resource requirements.** The host knows only generic resource names: Picture / Backdrop / Environment / History / View / Depth / Motion / Coverage / Target. An effect that fits within existing resource types can be added without a Rust build. Built-in effects follow the same contract as plugins.
- **Migration is destructive.** Once ownership moves, the old route is deleted. There are no fallbacks, no compatibility wrappers, and no 1:1 wrappers around Rerun APIs.

## Rulings that stand, and are not asked again

- **2.5D** is placed relative to the authored camera. The Stage observer looks from outside and does not billboard.
- **Plate** is an optimization intent plus semantic isolation. It is materialized only when pixels are needed. **Flatten** is a separate, explicit materialization.
- **Matte / Clip** are Motolii's coverage meaning. They do not reuse Plate's camera bake.
- **Standard Glass** reads a backdrop shared per view ([standard-glass-shared-transmission](2026-09-23-standard-glass-shared-transmission.md)).
- **Same-frame analysis:** the drawing route never reads the GPU back to the CPU within the same frame. Analysis readbacks go through the readback belt and arrive in a later frame. Only offline routes (export, freeze, tests) wait, and they wait in one place: the headless embedder's `wait_offline`.
- **Selection** uses geometry bounds.

## What must not come back

- `re_view_host`, `HostFrame`, and `SpatialStage` embedding.
- Motolii's own frame bookkeeping: pending lists, `flush_pending`, submit counters, mid-tick submits and waits, `read_texture_bytes`, `submit_within_frame` / `read_texture_now` / `schedule_*_in_order` / `after_submit_within_frame`.
- Motolii-owned GPU pools and caches: `EffectScratch`, `reflection_cache`, and the block program's own buffer retirement list and bind-group cache.
- Raw `device.create_*` outside the embedder.
- Pixel-tight selection bounds, the outline-only view, and the outline mask accessor.
- `Window.projection_camera`.
- Motolii shading semantics inside the fork: material, motion kinds, noise, `SceneReflection` / `SunLight` / `MotionBuffer` / `light_capture`.

## Result (static scan, final rules)

The scan was a throwaway script. It classifies the fork's public additions against upstream base `954bf95a4e`, and every GPU execution primitive in Motolii.

| | Before (fork `8496d38`, Motolii `77e3bef3d`) | After (fork `3650982120`, Motolii `2c40ce1cf`) |
|---|---|---|
| Fork public additions | 214: MOTOLII_SEMANTICS 20, UNCLASSIFIED 4 | 184, all UPSTREAMABLE |
| Fork WGSL functions added | 88 | 70 |
| Fork WGSL Motolii clusters (block motion, glass, reflection probes, sun cookie, extrusion, noise) | present in all six | 0 |
| Fork files containing `motolii` | 14 | 0 |
| Motolii HOST_VIOLATION | 56 | 0 (EMBEDDER 7, SEMANTIC_LOWERING 99) |

What remains on the fork's WGSL side is the generic contract: the field and surface hooks and the view near-plane fade.

The oracle has three parts, all run against the rev-pinned fork. motolii-render lib passes 323 and fails 8. motolii-ui lib passes 92 and fails 5. Every one of those failures predates the migration. The fork's `re_renderer` tests and the native Viewer build (`rerun-cli`, native_viewer) are green.

## On current upstream, with the old routes removed (2026-09-24)

The fork was reapplied onto current upstream main (`2309184bbb`, 590 commits past the old base). It is now branch `motolii/host-core-current` at `a8b897ef`, and Motolii is pinned to that rev. Upstream API changes were followed as migration, not redesign: `re_span::Span`, renderer lookups that return a `Result`, wgpu/naga 30, `Loggable` split into `ArrowDataType` + `To/FromArrow(Opt)`, sample sources, `FrameRetainedCache`, and focal length in the frame uniform.

Fork changes on top of the reapply:
- **Data textures for per-element data.** Upstream requests WebGL2-class device limits, which allow no storage buffers. The fork's storage buffers in the global and mesh layouts made those layouts invalid on the Viewer's device, and a build-only oracle could not see that. Motion and curve fills are now data textures, as upstream keeps per-element data (`motion_at` / `motion_len`).
- **`new_with_external_resolved`** only validates and imports the caller's texture. It now shares `new`'s body instead of copying it.
- **Removed:** transient attachments (wgpu 30 forbids storing them, and upstream's volume phase stores the MSAA target), the unused openh264 decoder, dead surface-program pipelines, and the outline-mask accessor.

The migration was held to completion conditions:
- **Fork:** 191 public additions, all UPSTREAMABLE. MOTOLII_SEMANTICS 0, unclassified 0, `motolii` in the fork diff 0.
- **Motolii:** HOST_VIOLATION 0.
- **Negative ownership audit:** the checkpoint's list of forbidden architecture was scanned as symbols across all production sources and the fork diff, with 0 hits. References to old fork revs: 0. Every Rerun crate resolves from the one new rev.
- **Old routes deleted, not left unused:** the pending video-frame copies, the texture route before FrameGraph (the text texture cache and friends), engine state nobody read, the tick's `not_ported` gate, and tests of removed owners. The compiler then built the workspace without them.
- **The Viewer at runtime:** re_renderer tests pass (64), and the native `rerun-cli` builds. The re_view_spatial snapshot tests match pure upstream: the same 94 fail on missing LFS assets, with no wgpu validation errors.

### The Plate / Glass fix found by the acceptance

In the real window, Glass Garden's petals were grey. The petals are glass inside a Repeater `.whole()` plate. Such a plate was still baked in the preparation over nothing (`NO_BACKGROUND`), so its glass found no backdrop and showed the environment.

A plate whose members read the backdrop is now `LayerContent::Plate`. The preparation prepares its members' pictures once. Each view draws the members at the plate's place in its stack, with that view's picture below as their backdrop, then runs the plate's effects on the drawing. Plates without glass keep their single bake per frame.

Two contracts pin this:
- Glass in a plate refracts the picture below it, and looks the same as the copies drawn each. On the old bake this renders grey.
- A plate without glass is baked once, however many views draw it.

A saved WGSL also now redraws a paused window: the effect catalog's generation is an input of the frame.

### Production acceptance: the existing Glass Garden, unmodified

- **Stage and Camera** show the same world as two views. In the Stage, the orbit line and the glass slab extend beyond the Camera's frame.
- **2.5D** follows the authored camera and does not billboard to the observer (contract `the_stage_window_draws_beyond_the_frame_and_the_camera_does_not`).
- **The glass flower** inside the Repeater plate is transparent and refracts the picture below it, in both views.
- **WGSL hot reload:** editing `vism/glass.wgsl` with no cargo build changed the paused window within seconds, and reverting restored it.
- **Stage ROI change:** the picture and its window/ROI matched within 1–2 frames (`PROBE room=stage-window`). An OS-level window resize could not be driven from background input and was not exercised.
- **Playback, 10 s:** both runs had two views (Stage 2443×1374 + Camera) at 60 fps.

  | Build | Frames drawn | Dropped | CPU submit (median / p90 / max, ms) |
  |---|---|---|---|
  | dev | 341 | 317 | 10.6 / 13.8 / 37.9 |
  | release (profile) | 354 | 307 | 8.8 / 10.9 / 51.4 |

- **Oracle:** motolii-render lib passes 322 and fails 8. motolii-ui lib passes 92 and fails 5. Both failure sets are the same as the baseline.
