# GPU Execution Graph Migration — ownership, residency, invalidation, lifetime

Status: **implementation canon / stacked on PR #506**  
Base semantic contract: [Stage 5 FrameGraph](frame-graph.md)

## Goal

Complete the GPU-side cutover after the semantic FrameGraph migration.

The end state is not merely "wgpu is used" and not merely "GpuScene was split".
The end state is:

- semantic evaluation and GPU execution have separate, explicit owners;
- unchanged GPU resources remain resident and are not rebuilt because time advanced;
- content and placement invalidate independently;
- effect / mask / matte / plate dependencies are explicit GPU-resource edges;
- texture / buffer lifetime is planned and reusable scratch storage is bounded;
- temporal GPU state (freeze / feedback / history) has explicit ownership;
- preview / Camera / Stage / export / headless share the same GPU execution graph and differ only at the sink;
- normal preview does not synchronously read back GPU output;
- performance is measured by work avoided as well as frame time;
- the former monolithic `GpuScene` execution boundary is removed.

This work keeps the Stage 5 rule:

> A semantic FrameGraph Node is a share/cache/invalidation/time boundary, not one GPU pass.

The GPU execution graph is therefore a lower-level execution/resource graph. GPU passes do not become semantic NodeKinds just because they exist.

## Current problem

Today the semantic graph is fine-grained, but GPU lowering crosses a coarse exact-time boundary:

```text
Semantic DAG
   ↓
GpuScene (Exact time)
   ↓
Vec<LayerWithPasses>
   ↓
Compositor
   ↓
sequential planning / temporary resources / draw / present
```

`GpuScene` owns whole-scene lowering. A time change invalidates this coarse boundary even when most layer content is unchanged. The compositor then owns execution decisions and resource lifetime outside the graph.

This collapses much of the upstream invalidation precision at the GPU boundary.

## Target architecture

```text
AUTHORING
Layer / Group / Effect / Mask / Matte
        ↓
SEMANTIC DAG
Property / Transform / MediaFrame / Geometry / Effect / Mask / Matte / Scene
        ↓
GPU LOWERING
GpuContentResource
GpuGeometryResource
GpuPlacement
GpuEffectInputs
        ↓
GPU RESOURCE GRAPH
resource dependencies
resource versions
texture / buffer lifetime
history / freeze / feedback
        ↓
GPU EXECUTION PLANNER
render / compute work
pass fusion
resource aliasing
composition ordering
        ↓
EXECUTOR
wgpu / re_renderer
        ↓
SINK
preview | stage | export | headless
```

## Ownership rules

### Semantic graph owns

- authored meaning;
- evaluated transforms, effect values, mask/matte meaning;
- temporal dependencies;
- visibility semantics;
- ordering semantics;
- exact media-frame identity;
- group / plate meaning.

### GPU lowering owns

- conversion of semantic values into reusable GPU-resident resources;
- content identity and resource versioning;
- placement/uniform updates;
- GPU-side dependency edges needed by effects, masks, mattes and plates.

### GPU resource graph owns

- resource dependency and liveness;
- resource cache hit/miss;
- scratch lifetime;
- aliasing/reuse eligibility;
- temporal resource retention;
- generation retirement.

### GPU execution planner owns

- which GPU work is needed for the current sink;
- pass ordering;
- pass fusion opportunities;
- render/compute scheduling;
- final composition plan.

### Executor owns

- recording/submitting already-decided GPU work;
- no hidden document reads;
- no whole-scene semantic decisions.

### Sink owns

- presentation to the window/surface;
- export/readback/encoder handoff;
- headless/test capture.

A sink must not cause upstream semantic or GPU-resource reevaluation unless its actual required output differs.

## Required boundaries

### 1. Content vs placement

A transform-only change must not rebuild texture/mesh/point-cloud/vector content.

```text
GpuContentResource
  texture / mesh / point cloud / vector backing
          ↓
GpuPlacement
  transform / opacity / z / projection / blend parameters
```

Content is long-lived where possible. Placement is cheap and may change every frame.

### 2. Layer contribution vs whole scene

Whole-scene `GpuSceneValue { Vec<LayerWithPasses> }` is a migration shim only.
The graph must eventually hold independently reusable GPU contributions.

### 3. Effects / mask / matte / plate

Dependencies are explicit. Changing an effect on layer B does not rebuild unrelated layer A content.
A matte source must be represented as a dependency edge, not recovered by a hidden whole-scene scan.

### 4. GPU temporal state

Freeze, feedback and history retain explicit GPU results with bounded ownership and generation rules.
They must not rely on accidental compositor-local history.

### 5. Final projection / sink

Camera, Stage and export share upstream GPU resources.
Only final projection or output sink may differ when the semantic inputs are the same.

## Migration waves

### G0 — Inventory and observability

Before changing behavior, account for every current responsibility in `GpuScene` and compositor:

- media decode/upload;
- text/shape raster/tessellation;
- mesh/point-cloud resources;
- effect inputs and passes;
- masks;
- mattes;
- clip;
- flatten / Group plate;
- Block / overlay;
- selection outline;
- composition;
- present;
- readback.

For each responsibility record:

- semantic owner;
- GPU resource owner;
- invalidation source;
- time dependency;
- output resource;
- cache lifetime;
- readback behavior.

Add counters/measurement sufficient to prove which GPU resources were rebuilt.

### G1 — Per-contribution GPU boundary

Introduce a per-contribution GPU resource boundary under the semantic graph.

Exit condition:

- one layer/contribution can miss while unrelated contributions hit;
- `GpuScene` no longer performs mandatory whole-scene resource recreation;
- pixel semantics are unchanged.

### G2 — Split content from placement

Separate expensive GPU content from cheap per-frame placement/uniform state.

Required regression:

```text
10-layer scene
change only Layer 5 transform

expected:
semantic invalidation: Layer 5 transform chain
GPU content rebuilds: 0
GPU placement updates: 1 contribution
final composition: required
```

### G3 — Explicit effect/mask/matte/plate resource edges

Move dependency discovery out of monolithic scene lowering.

Required regressions:

- changing one effect rebuilds only its dependent chain;
- matte source changes rebuild only consumers and downstream composition;
- Group Whole/flatten plate invalidates only its plate subtree;
- clip and stencil semantics remain pixel-identical.

### G4 — GPU execution planner

Compositor stops being the place that discovers what work exists.
It executes a prepared plan.

The plan describes:

- required resources;
- required passes;
- ordering;
- target/sink;
- resource use intervals.

Semantic NodeKind count remains independent from GPU pass count.

### G5 — Resource lifetime and aliasing

The resource graph computes last use and returns scratch resources to bounded pools.

Exit condition:

- temporary texture creation is not proportional to unchanged layer count per frame;
- compatible non-overlapping scratch resources can reuse backing storage;
- peak bytes and allocation count are measurable.

### G6 — Temporal GPU cache

Integrate explicit history/freeze/feedback GPU ownership.

Exit condition:

- previous-frame results can remain GPU-resident;
- seek/backward replay is deterministic;
- eviction is bounded and observable;
- no hidden compositor-local temporal owner remains.

### G7 — Shared sinks

Preview, Camera, Stage, export and headless consume the same upstream GPU graph.

Allowed divergence is only at the actual sink:

```text
shared GPU result
   ├─ present(surface)
   ├─ export(readback/encoder)
   └─ test(readback)
```

Normal interactive preview must not wait for synchronous readback.

### G8 — Performance closure

Do not close this migration because the architecture "looks right".
Close it only with representative playback evidence.

Measure at minimum:

- semantic prepare CPU time;
- GPU-lowering CPU time;
- resource cache hit/miss;
- GPU resource creations;
- scratch checkouts/reuse;
- queue submissions;
- synchronous waits;
- readbacks;
- GPU frame time;
- end-to-end preview frame time;
- memory/VRAM high-water mark where observable.

## Performance acceptance fixtures

At least these workloads must be covered.

### Static-heavy scene

Many unchanged image/text/shape layers while time advances.

Expected: unchanged GPU content remains resident; exact time alone does not rebuild all resources.

### Transform drag

One layer moves interactively.

Expected: content rebuild 0 for that layer; unrelated resource rebuild 0; placement/update path dominates.

### Single-effect edit

One effect parameter changes on one layer.

Expected: only the dependent GPU effect chain and composition invalidate.

### Video + static overlays

Video frame advances under static graphics.

Expected: video content changes; static graphics remain GPU cache hits.

### Matte / Group Whole

Change matte source or one child under a Group Whole boundary.

Expected: only the actual dependent plate subtree invalidates.

### Scrub backward/forward

Expected: deterministic visual result; temporal resources obey explicit generation/history policy.

## Hard gates

The migration is not complete until all are true:

- [ ] no monolithic exact-time `GpuScene` resource owner remains;
- [ ] GPU content and placement have independent invalidation;
- [ ] unrelated layer GPU resources survive edits and time advancement;
- [ ] effect/mask/matte/plate dependencies are explicit;
- [ ] compositor is an executor, not the hidden execution planner;
- [ ] GPU resource lifetime is represented and bounded;
- [ ] scratch reuse/aliasing is measurable;
- [ ] temporal GPU resources have explicit ownership;
- [ ] preview/export/headless share the same upstream execution graph;
- [ ] interactive preview synchronous readback/wait = 0 on the normal path;
- [ ] representative fixtures prove avoided work, not only identical pixels;
- [ ] representative playback is materially faster than the pre-migration baseline;
- [ ] GPU parity / native validation is rerun on the supported native backend before merge.

## Non-goals

- one semantic Node per GPU pass;
- replacing wgpu/re_renderer merely for architectural purity;
- changing document meaning to fit the renderer;
- accepting whole-scene invalidation because a small fixture is fast;
- claiming completion from microbenchmarks alone;
- making export readback disappear when a CPU encoder genuinely requires bytes.

## PR strategy

This branch is intentionally stacked on PR #506 so the GPU migration starts from the completed semantic ownership cutover.

The PR remains draft while waves are incomplete. Commits should preserve working pixel semantics and make each ownership transfer explicit. When a wave is too large to review safely, it may be split into a child PR, but the completion checklist above remains the canonical closure gate.

Native GPU execution that cannot run in GitHub-only editing is a final verification gate, not a reason to defer ownership cleanup.
