# GAP-30 cold pipeline: inventory and measurement (2026-09-28)

Step 1 of GAP-30 only: every product pipeline, who makes it on which thread, and what a cold first frame costs.
The capture surface, the owner and the descriptor closure are **not** decided here (see "Open" below).

## Who makes pipelines, on which thread

There is no Motolii `PipelineCache`. The owner is `Engine` → `Compositor` → the forked `re_renderer::RenderContext`
(`oshikaidesu/rerun@d046cd9`); its `StaticResourcePool` keys render pipelines by the full descriptor (label, layout,
shader modules, entry points, target formats), never evicts, and passes `cache: None` (nothing on disk, nothing
across processes).

| Where | What | Cache | When |
|---|---|---|---|
| `compositor/selection_bounds.rs:66,92` | selection bounds compute (readback) | direct wgpu | once, eagerly, in `Compositor::with_device` |
| `compositor/effects/vism.rs:186-239` via `LazyEffectProgram` / `render_effects.rs:58-83, 273-282` | Vism/ISF effect passes, blend/matte built-ins, encoding conversion, matte coverage per format | upstream pool + Motolii maps (`effect_programs` by plugin id, `coverage_programs` by format) | lazily, the first frame a plugin/format appears |
| `compositor/effects/surface_program.rs:175` | 3D surface / field shading | `surface_programs` by `unlit|field|surface|catalog generation` | lazily |
| `compositor/effects/block_program/passes.rs:54-58` | block simulation compute (Block/Follow/World/Rope passes) | direct wgpu, rebuilt on source change | the first frame with blocks |
| upstream renderers (`ctx.renderer::<R>()`), mipmap, YUV→RGB | rectangles, meshes, paths, lines, points, skybox, outlines, compositor, mip, YUV | upstream pool | lazily, once per context |

Threads: the editor's `Engine` lives on Swift's serial `motolii.render` queue (`EditorRuntime::open` →
`motolii_probe_open`); **export** (`ui/extensions/jobs/src/export.rs:106-111`) and **freeze**
(`ui/extensions/jobs/src/freeze.rs:132-137`) each run their own `Engine::new()` on their own thread — a new
`RenderContext`, so every pipeline is compiled again for them.

## Measured (Apple M4, Metal, release; `crates/motolii-render/examples/cold_first_frame.rs`)

| document | first frame, OS/driver cold (1st run after build) | first frame, process cold (runs 2–3) | warm, same frame | warm, next frame | 2nd `Engine` in the same process, first frame |
|---|---|---|---|---|---|
| `field-2d/outlined-text.rrd` | 1285 ms (+107 ms `Engine::new`) | 85 ms | 6.3 ms | 6.4 ms | 64 ms |
| `glass-gallery/light-in-form.rrd` | 301 ms | 200 ms | 11.8 ms | 17–20 ms | 187 ms |
| edit fixture (`fixture::build`) | 489 ms | 51 ms | 6.0 ms | 6.2 ms | 46 ms |

Reading:
- The very first run after a build pays the driver's shader compile (Metal keeps it across processes): up to 1.3 s.
- Every new process pays 50–200 ms on its first frame, 8–17× a warm frame.
- Each `Engine::new()` makes its own instance and device (`compositor/headless.rs`); after the first in a process it
  takes ~0.7 ms. A second `Engine` pays the first-frame cost again (46–187 ms) — what export and freeze pay.
- **Not isolated here:** how much of a first frame is pipeline creation versus other first-use work on a fresh
  context (glyph atlases, image and media uploads, buffer allocation). The next step is to time pipeline creation
  itself (around the pools' create calls) before choosing a prewarm or a shared context.

## Open (decisions, not taken here)

- Where a prewarm/capture lives (Engine, Compositor, the fork's pools) and whether export/freeze share the
  editor's pipelines (one `RenderContext` across threads) or keep their own — a threading/ownership choice.
- Whether to persist pipeline binaries (`wgpu::PipelineCache`) across processes, and its invalidation key.
- The first-display SLO (what "not stopped" means for the UI) and pending/last-good behaviour on compile failure.
