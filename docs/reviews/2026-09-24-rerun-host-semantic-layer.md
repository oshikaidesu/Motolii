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

The oracle has two parts. motolii-render lib passes 323 and fails 8, and the 8 failures all predate the migration. The fork's `re_renderer` tests and the native Viewer build (`rerun-cli`, native_viewer) are green.
