# Render orchestration is Rerun's (2026-09-23)

Ruling by the user (2026-09-23): Motolii is "an AE built on Rerun". Being Rerun-based does not just mean using the `re_renderer` crate to draw. **Rerun/re_renderer's execution model is the standard for renderer infrastructure.**

The old orchestration tangled five things together, and each fix added more branches: view ownership, the frame lifecycle, world preparation, caches, and fallbacks. So the orchestration alone is **rewritten from a minimal structure** along Rerun's actual usage.

The renderer's features are kept as they are: FrameGraph, lowering, the re_renderer fork, shape/text, materials, WGSL, effects, plates, glass/reflection, matte, ordered transmission, and the tests.

The old path is **not an architectural reference**. It is used only as the oracle for behaviour and pixels.

## Who owns what

| Motolii owns (film semantics) | Rerun/re_renderer owns (infrastructure; follow its actual usage) |
|---|---|
| FrameGraph and the meaning of the work, time evaluation, Group/Repeater/Layout, Effect, Matte/Mask, Plate/Backdrop, ordered transmission/compositing, Feedback/Freeze, JS/Live, authoring semantics | App frame lifecycle, where `begin_frame`/end of frame fall, multi-view, `ViewBuilder` lifecycle, GPU resource lifetime, staging belts, pools, frames in flight, submit/flush, mesh/material/texture reuse and upload, render targets, generic batching/instancing |

**Departing from Rerun's model requires stating which of Motolii's film semantics needs it.**

For a feature Rerun doesn't have (plate, ordered transmission, feedback), first look for a way to insert it on top of Rerun's frame/resource/view lifecycle without breaking it. Building a lifecycle of our own is the last resort.

## One tick in Rerun (the call graph)

The fork's embedder `SpatialStage::show` (`re_view_spatial/src/spatial_stage.rs:321-455`) does the same as the viewer (`re_viewer/src/app/ui.rs:109,168`):

1. `store_hub.begin_frame_caches` / `app_caches.begin_frame`: the cross-frame caches mark a new generation.
2. **`render_ctx.begin_frame()` ×1** (`spatial_stage.rs:333`; `re_renderer/src/context.rs:495`). It:
   - submits the leftover frame-global encoder;
   - recycles the belts;
   - advances the frame index;
   - ages the pools (a resource unused for a frame is destroyed at the next `begin_frame`);
   - waits while more than 4 submissions are in flight.
3. `run_once_per_frame_context_systems`: **shared preparation ×1** for all views.
4. For each view:
   - `execute_systems_for_view`, then `class.ui`, which builds that view's `ViewBuilder` (resolution = rect × ppp);
   - the draw data (Mesh/Rect/Line) is per view, while the mesh cache and textures are shared.
5. egui-wgpu:
   - `prepare`: `ViewBuilder::draw` (`re_viewer_context/src/gpu_bridge/re_renderer_callback.rs:28-51`) returns a command buffer;
   - `paint`: `ViewBuilder::composite` (`:53-89`) goes into the surface's pass.
6. **`render_ctx.before_submit()` ×1** (`spatial_stage.rs:450`), then egui makes **one `queue.submit`** for everything.

## Motolii's tick

`Engine::tick(doc, time, views)` (`motolii/crates/motolii-render/src/engine/frame_graph/tick.rs`) does, in order:

1. `begin_frame` ×1.
2. **Prepare** the document frame ×1: FrameGraph evaluation → lowering → prepared GPU scene. Density comes from the views passed to this tick and is fixed before preparing.
3. For each view: `ViewBuilder::draw` → `composite` into the surface.
4. `before_submit`, then **one submit**.

**Where a piece of work belongs is decided by its inputs**, not by where the old code put it:

| What the work reads | Where it goes |
|---|---|
| The document frame (evaluation, lowering, geometry, materials, a layer's own effect chain, plates, environment, world reflection, light cookie, block motion) | prepare, once per document frame |
| A view's projection, visibility, LOD or density request | the View |
| A view's own image or history (screen-space effects, the part of ordered transmission that reads the view's image, window feedback, outline) | the View's render and history |

**World captures place 2.5D layers by the output's camera.**
- 3D placement does not depend on the camera (`layer_projection_transform` is the identity for 3D). 2.5D placement follows the camera.
- The world's reflection and light cookie are made once and do not depend on any view. They therefore place 2.5D layers with the document camera, which is the output's world.
- A Stage view only looks at that world. When Stage draws a 2.5D layer itself, it places it by its own camera, but what a mirror reflects is the output's placement.
- The Camera view and export pictures are the same as before.

## Rerun / Motolii / reason for any difference

| Topic | Rerun's actual usage | Motolii's new path | Reason for a difference |
|---|---|---|---|
| begin_frame | Once per app frame, before all views | Once at the start of `tick` | None |
| Shared preparation | Once per frame, before the views | prepare ×1; not re-prepared when revision, time and density are unchanged | None |
| Views | Per-view ViewBuilder; draw → composite | Same | None |
| Submit | One per frame (egui) | One per tick | None. Readback and plates will be inserted as passes inside the tick. |
| Density / resolution | rect × ppp | Views request it; it is fixed before prepare | Plates and captures are world-owned and must be baked at the densest view's density (the density law) |
| Pools / belts / lifetime | re_renderer's own pools and belts | re_renderer's, used as they are | Motolii's own pools will be justified one by one as features are ported |

## Invariants (regression tests)

`engine/frame_graph/tick_tests.rs` checks:
- `begin_frame == 1` per tick;
- `submit == 1` per tick;
- preparation ≤ 1 per document frame, and 0 when the same frame is shown again;
- with 0, 1, 2 or 5 views, only the view count changes;
- every view reads the same prepared frame (the Arc is identical).

Tests to add as features are ported:
- world captures and resources don't grow with the view count;
- shared mesh/material uploads are not repeated;
- views don't mutate the world;
- frame-scoped resources are not retired at a view boundary.

**If the same boundary needs more than one cache, reuse or special case, stop patching and re-audit the ownership boundary first.**

## Porting order

The old path is the pixel oracle. Each feature is compared against the old path's output.

1. Background (done)
2. Rectangle layers
3. Paths/text
4. Meshes and materials
5. Effects (a layer's own chain)
6. Plates
7. Matte/clip
8. Reflection and light cookie
9. Ordered transmission (backdrop)
10. Screen-space effects and outline
11. Feedback/Freeze
12. UI (one FFI tick for all views)
13. Glass Garden

Once the new path draws Glass Garden, the old orchestration is deleted. There is no compatibility shim.
