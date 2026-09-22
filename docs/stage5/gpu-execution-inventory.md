# GPU execution ownership inventory

Status: **G0 inventory for GPU execution graph migration**  
Branch: `gpt/gpu-execution-graph-20260922`

This inventory records the current GPU-side responsibilities that must be separated without changing pixel semantics.

| Responsibility | Current owner / route | Current invalidation shape | Target owner | Migration note |
|---|---|---|---|---|
| Whole-scene GPU lowering | `engine/frame_graph.rs::NodeKind::GpuScene` → `prepare_gpu_scene_with_solver` | exact-time whole scene | per-contribution GPU lowering | remove monolithic exact-time resource boundary |
| Scene culling | `engine/frame_graph_scene.rs::plan_frame_graph_scene` | recomputed while lowering scene | GPU execution planner | execution-only; semantic SceneValue remains unchanged |
| Text/shape GPU content | `engine/frame_graph_scene.rs::prepare_gpu_scene` plus existing raster/tessellation helpers | reached through whole-scene lowering | `GpuContentResource` | content identity must survive placement-only changes |
| Still/video GPU content | `prepare_gpu_scene` + media/GPU frame cache | reached through whole-scene lowering; video source frame may change | `GpuContentResource` keyed by exact media frame | advancing video must not invalidate static overlays |
| Mesh / point-cloud content | `prepare_gpu_scene` + compositor resource helpers | reached through whole-scene lowering | reusable geometry/content resource | placement must be independent from imported geometry |
| Placement / transform / opacity / projection | materialized into `LayerWithPasses` during scene preparation | coupled to content lowering | `GpuPlacement` | transform-only edit must rebuild zero GPU content |
| Effect-chain image production | `compositor/render_effects.rs::effective_layer_textures[_in_frame]` | compositor discovers and creates effective outputs | GPU resource graph + execution planner | preserve existing effect sharing while making dependency/lifetime explicit |
| Screen-space passes for non-planar content | `compositor/sequential/accumulate.rs::apply_screen_passes` | discovered inside sequential run | execution planner | keep semantic effect node independent from pass count |
| Matte execution | semantic matte dependency lowered into compositor matte implementation | dependency partly materialized through prepared layer list | explicit GPU resource edge | source changes invalidate only actual matte consumers |
| Clip / stencil | scene lowering plus compositor matte/clip implementation | coupled to prepared scene ordering | explicit dependent contribution/plate edge | preserve existing pixel contracts |
| Group Whole / flatten plate | existing isolated bake / plate lowering | plate work reached through scene lowering | explicit plate resource node/edge below semantic graph | only plate subtree should invalidate |
| Block / Follow / field / physics GPU state | `engine/blocks.rs::prepare_frame_graph_blocks` using evaluated Scene + Solver only | invoked from `prepare_gpu_scene_with_solver`; physics path can cause two scene preparations | dedicated solver/block GPU resource owner | must not restore StoreView reads; remove duplicated whole-scene lowering |
| Freeze cached content | `frame_graph_scene.rs::frame_graph_frozen_content` | checked during per-source lowering but owned under monolithic GPU scene call | temporal GPU resource cache | cache policy, not semantic FreezeNode |
| Composition ordering | `compositor::sequential_inputs` → `accumulate_sequential` | rebuilt from layer slice per render | GPU execution planner | planner decides work; executor records/submits |
| Scratch texture lifetime | compositor effect scratch / blend scratch | local/manual checkout and release | GPU resource graph | expose last-use, bounded reuse and aliasing |
| Final preview target | `compositor/presentable.rs::render_into_window` → `finalize_into` | sink-specific target | sink | upstream GPU resources stay shared |
| Export/headless readback | `sequential/finalize.rs::finalize_readback` and test/headless paths | explicit GPU wait/readback | export/headless sink | readback is allowed here; not a preview-path owner |
| Selection bounds readback | `compositor/selection_bounds.rs`; async/nonblocking path exists for interactive use | only selection/outline dependency should trigger it | auxiliary sink/readback | must not become a global frame wait |
| Queue submit / final flush | compositor sequential/finalize paths | executor-local | GPU executor | planner should decide work before submit |

## Confirmed coarse boundary

Current production graph creates one exact-time GPU node:

```text
Scene + Camera + Solver + Overlay
            ↓
        GpuScene
   (TimeDependency::Exact)
            ↓
  GpuSceneValue {
    Vec<LayerWithPasses>,
    Vec<LayerId>
  }
```

This is the first ownership boundary to dismantle. It is not sufficient to rename it or wrap each vector entry after the fact: per-contribution identity and invalidation must exist before expensive GPU resource creation.

## Existing behavior that must be retained

The migration reuses proven implementation pieces rather than rewriting them for purity:

- current still/video decode and GPU-frame cache;
- current text/shape raster/tessellation;
- current mesh/point-cloud resources and supported instancing;
- current effect shaders and effect-chain semantics;
- current matte/clip/stencil/flatten pixel contracts;
- current re_renderer/wgpu presentation route;
- current no-synchronous-final-readback interactive path;
- current FrameGraph rule forbidding hidden StoreView/document reads below evaluated semantic inputs.

## G1 first cut

The first behavioral refactor should introduce an independently identifiable GPU contribution before composition.

A contribution needs enough identity to answer these questions without scanning/rebuilding the whole scene:

1. Did the expensive content change?
2. Did only placement/uniform state change?
3. Which effect/mask/matte/plate resources depend on it?
4. Can the previous GPU resident resource be reused?
5. Is the contribution required for this sink/view?

The first cut may retain `LayerWithPasses` as an adapter payload while ownership moves. `LayerWithPasses` itself must not remain the long-term whole-scene cache key.

## G1 acceptance

A focused architecture/performance fixture must be able to prove:

```text
Given 10 visible static contributions
When only contribution 5 position changes

GPU content rebuilds for contributions 1..10 = 0
placement/resource-state update for contribution 5 = 1
unrelated contribution invalidations = 0
final composition = required
pixel output = unchanged from baseline
```

A second fixture must prove:

```text
Given video contribution A + static overlay B
When time advances one frame

A media/content identity = changes as required
B GPU content identity = stable
B content rebuild = 0
composition = required
```

These are ownership tests, not merely timing benchmarks.
