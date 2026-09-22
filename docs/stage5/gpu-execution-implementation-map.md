# GPU execution implementation map

Status: **implementation map for PR #511**  
Decision: destructive GPU control-plane rewrite; old orchestration is not a compatibility target.

## Core rule

Do not optimize the old owner. Replace it.

The semantic FrameGraph remains the source of truth. The new GPU control plane consumes per-contribution semantic values and versions, maintains GPU residency, plans only required work, and hands an immutable plan to the backend executor.

## Control-plane modules now fixed

`src/gpu_exec/types.rs`
- stable logical resource identity
- resource value version
- lifetime policy
- pass identity
- alias compatibility

`src/gpu_exec/graph.rs`
- persistent logical GPU resource graph
- precise downstream invalidation
- residency state
- producer ownership
- bounded temporal retirement

`src/gpu_exec/planner.rs`
- cache-hit cuts
- pass reachability/order
- resource liveness
- transient alias assignment

`src/gpu_exec/executor.rs`
- backend-only execution boundary
- executes an already-built plan
- marks successful outputs resident
- never discovers dependencies

`src/gpu_exec/lowerer.rs`
- per-contribution semantic → GPU logical-resource seam
- no StoreView
- no whole-scene owner
- no command submission

## Critical versioning decision

Many current semantic nodes are marked `TimeDependency::Exact` even when their evaluated value remains unchanged (Transform, WorldTransform, Effect, Mask, etc.).

Therefore:

**GPU resource version must never be derived from comp time, generation, or “semantic node executed this frame”.**

It is derived from a canonical fingerprint of the **GPU-visible evaluated value**.

Examples:

- static transform at t=0 and t=1 → same placement resource identity **and same version**
- animated transform whose matrix changed → same identity, new version
- video frame changed → same media-content identity, new version
- static overlay while video advances → overlay identity/version unchanged

`GpuResourceVersion::from_canonical` exists specifically for this rule.

## Semantic adapter work

The next adapter must expose per-contribution semantic bindings without rebuilding SceneValue as the GPU owner.

Required source bindings per contribution:

| GPU input | semantic source |
|---|---|
| contribution identity | `CompositeContribution` NodeKey |
| content identity/value | TextShape / ShapeGeometry / Material / MediaFrame / Particle / Plate source |
| placement | WorldTransform / placement sample |
| effects | evaluated Effect NodeKeys + values |
| masks | evaluated Mask NodeKeys + values |
| matte relation | explicit source contribution/resource edge |
| plate | GroupComposite / motion/placement plate value |
| ordering | SceneComposite ordering only; not a resource identity substitute |

### Minimal semantic API additions

Expose from `SceneProgram` / `SceneNodeProgram`:

- contribution binding by LayerId
- contribution bindings iterator
- EffectProgram accessor
- MaskProgram accessor

These are compiler/runtime binding lookups only. They do not make LayerId a GPU cache identity.

## Cutover sequence

### C1 — semantic adapter

Files:
- `frame_graph/scene_program.rs`
- `frame_graph/program.rs`
- new `gpu_exec/semantic_adapter.rs`

Deliver:
- per-contribution `GpuContributionInput`
- canonical versions for content / placement / effects / masks
- no GPU work yet

Gate:
- static transform version unchanged across different comp times
- animated transform changes placement version only
- static content version unchanged across unrelated time changes

### C2 — content lowering

Move content resource ownership out of:
- `engine/frame_graph_scene.rs::prepare_gpu_scene`

Reuse implementation primitives:
- text/shape texture builders
- still/video GPU frame cache
- mesh / point-cloud import
- particle GPU representation

Deliver:
- `GpuResourceClass::Content`
- upload/build producer passes
- resource-store backend handles

Gate:
- 10 static layers, time advances → content producer passes executed = 0
- video + 9 static layers → only video content producer executes

### C3 — placement lowering

Move placement out of whole-scene `Layer` construction.

Deliver:
- `GpuResourceClass::Placement`
- transform/opacity/z/projection/version data
- content resource does not contain placement state

Gate:
- transform drag → content rebuild = 0
- unrelated placement updates = 0

### C4 — effect/mask chain

Move effect/mask execution discovery from:
- `compositor/render_effects.rs::effective_layer_textures[_in_frame]`

Reuse:
- existing EffectPass/WGSL/ISF low-level executor initially

Deliver:
- explicit Effect and Mask resource edges
- chain output resource
- cacheable pass descriptions

Gate:
- one effect parameter edit invalidates only that chain and consumers

### C5 — matte/clip/stencil/plate

Move dependency discovery from:
- `engine/frame_graph_scene.rs` whole-scene scans
- compositor-local matte/clip decisions

Deliver:
- explicit matte source edge
- clip/base edge
- Group Whole / flatten plate resource
- dependent subtree invalidation

Gate:
- source change invalidates target chain only
- unrelated layers remain current

### C6 — composition planner

Replace:
- `sequential_inputs`
- `accumulate_sequential` as work-discovery owner

Keep:
- low-level draw implementations as backend operations

Deliver:
- composition pass DAG
- explicit ordering edges
- final composite resource

Gate:
- compositor backend receives plan; it does not scan semantic layers to decide work

### C7 — liveness / aliasing

Replace local scratch ownership with planner lifetime.

Targets:
- effect scratch
- blend scratch
- temporary plate surfaces

Deliver:
- concrete alias classes from format/size/usage/sample count
- physical scratch pool keyed by planner slot

Gate:
- non-overlapping compatible scratch resources share allocation
- allocation count / high-water bytes reported

### C8 — temporal state

Move:
- feedback history
- Freeze GPU residency
- previous-frame outputs

into explicit Temporal resources.

Gate:
- bounded history
- deterministic seek/replay
- no compositor-hidden temporal owner

### C9 — sinks

Final shared upstream graph:

```text
GPU composite result
  ├─ Present
  ├─ Export Readback
  └─ Headless Readback
```

Gate:
- preview normal path synchronous GPU wait/readback = 0
- export/headless divergence starts only at sink pass

### C10 — delete old orchestration

Delete production ownership:
- `NodeKind::GpuScene`
- `GpuSceneValue`
- `prepare_gpu_scene_with_solver`
- `prepare_gpu_scene`
- whole-scene GPU culling owner in `frame_graph_scene.rs`
- compositor execution-discovery paths superseded by planner

Retain old code only when it is a low-level primitive called by the new backend.

No fallback to old GPU orchestration.

## Performance counters required before closure

New control plane must expose per frame:

- resource inserts
- version changes
- resident hits
- producer passes executed
- producer passes skipped due residency
- total passes executed
- transient logical resources
- physical scratch slots
- aliased resource count
- estimated persistent bytes
- estimated transient high-water bytes
- queue submissions
- synchronous waits
- readbacks
- CPU lowerer/planner/executor time
- GPU timestamp time where supported

## Closure fixtures

1. 100 static layers, playback for N frames
   - content producer after warm frame: 0/frame
2. one-layer transform drag in 100-layer scene
   - content rebuild: 0
   - placement update: 1
3. video + static overlays
   - video content update: 1
   - static content update: 0
4. single effect parameter edit
   - only dependent effect chain invalidates
5. matte source edit
   - source + consumers only
6. Group Whole child edit
   - plate subtree only
7. forward/back scrub
   - deterministic output; bounded temporal resources
8. Preview vs Export
   - same upstream GPU graph; sink differs only at Present/Readback

## What Instant should not redesign

The following decisions are frozen for implementation:

- semantic graph and GPU graph use different vocabularies
- resource identity and resource version are separate
- GPU value versions are value-derived, not frame-derived
- content and placement are independent resources
- compositor/backend executes a plan; it does not discover one
- transient aliasing is based on planner liveness
- old `GpuScene` orchestration is deleted, not retained as fallback
