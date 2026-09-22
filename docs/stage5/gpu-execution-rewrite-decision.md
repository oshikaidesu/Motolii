# GPU execution rewrite decision

Status: **adopted — destructive migration, performance first**  
Date: 2026-09-22  
PR: #511

## Decision

Do not preserve compatibility with the current GPU orchestration layer.

The fastest path is a **greenfield GPU control plane under the existing semantic FrameGraph**, while reusing proven GPU implementation primitives.

### Keep

- the semantic/evaluation DAG from PR #506;
- wgpu device/queue/context ownership;
- re_renderer draw primitives that already express Motolii output correctly;
- media decode and GPU frame upload/cache primitives;
- text/shape rasterization and tessellation;
- mesh / point-cloud import and GPU representations;
- existing effect WGSL/ISF programs and low-level effect execution helpers;
- existing matte/clip/flatten kernels when their semantics are correct.

### Replace

- `GpuScene` as a whole-scene exact-time owner;
- `GpuSceneValue { Vec<LayerWithPasses> }` as the execution boundary;
- compositor-local discovery of required work;
- per-frame reconstruction of unchanged layer GPU state;
- hidden scratch/resource lifetime decisions inside sequential composition;
- any architecture whose cache key is effectively “current frame + whole scene”.

## New architecture

```text
Semantic FrameGraph
        ↓
GpuLowerer
        ↓
GpuResourceGraph
        ↓
GpuPlanner
        ↓
GpuExecutionPlan
        ↓
GpuExecutor
        ↓
Sink
```

### GpuLowerer

Consumes evaluated semantic values and produces stable GPU resource descriptors.

It does not submit work and does not own a whole scene.

### GpuResourceGraph

Owns:

- stable resource identity;
- content vs placement separation;
- dependency edges;
- liveness;
- temporal retention;
- invalidation;
- residency;
- scratch reuse eligibility.

### GpuPlanner

Owns:

- reachability for the requested sink;
- render/compute pass ordering;
- pass fusion;
- composition ordering;
- resource use intervals;
- aliasing opportunities.

### GpuExecutor

Owns only:

- recording commands;
- submitting commands;
- presenting/copying to the selected sink.

It does not discover semantics or dependencies.

## Required first-class identities

At minimum:

- `GpuContentKey`
- `GpuPlacementKey`
- `GpuEffectKey`
- `GpuMaskKey`
- `GpuMatteKey`
- `GpuPlateKey`
- `GpuHistoryKey`
- `GpuResourceId`
- `GpuPassId`

Exact names may change; the ownership split may not.

## Performance rule

A semantic change may invalidate only the GPU work that consumes it.

Examples:

- transform change → placement/composition only;
- video frame advance → that media content + consumers;
- effect parameter change → that effect chain + consumers;
- matte source change → source + matte consumers;
- unrelated static layers remain resident.

Advancing composition time by itself is **not** a reason to rebuild static GPU content.

## Migration policy

No compatibility shim is required for the old GPU orchestration.

Temporary side-by-side code is allowed only to keep the branch buildable while replacing sections. Once a new owner is live, the old owner should be deleted rather than retained as fallback.

Pixel parity tests remain useful as semantic oracles, but old internal structure is not an oracle.

## Completion condition

The rewrite is complete only when:

1. `GpuScene` and whole-scene GPU lowering are deleted from production;
2. compositor/sequential no longer owns execution discovery;
3. resource lifetime/residency/invalidation are explicit;
4. static GPU resources survive time advancement and unrelated edits;
5. preview/export/headless share one upstream plan;
6. representative workloads show materially lower CPU preparation cost, resource creation count, and end-to-end frame time.
