# Final GPU cutover freeze

This document freezes the last architectural decision before deleting the old GpuScene orchestration.

## 1. Final ownership

The semantic FrameGraph owns meaning and evaluated values.

The GPU execution graph owns:
- resource identity/version/lifetime,
- explicit dependencies,
- execution ordering,
- transient liveness/aliasing,
- temporal snapshots and history identities,
- SceneComposite reachability,
- Present and Readback sink selection.

The compositor remains as a low-level GPU primitive library and command recorder.

It must not own semantic work discovery after cutover.

## 2. Deferred SceneComposite

SceneComposite is NOT required to be a full-frame texture.

Motolii mixes 2D, 3D, point clouds, meshes, environment, backdrop-reading materials and window-specific projection. Baking a single view-independent texture before the sink would either lose those semantics or duplicate Stage/Camera work.

Therefore the final upstream root is:

```
ordered contribution outputs
          |
          v
deferred SceneComposite
       /        \
 Present       Readback
```

The SceneComposite root fixes ordered reachability and dependencies. The sink supplies view-specific camera/window/background/output requirements.

Present and Readback share the exact same upstream resource graph.

## 3. What moves out of Compositor

These are planning/discovery and move to gpu_exec:

- deciding which effect outputs are needed;
- turning layer/effect outputs into ordered composition inputs;
- deciding run boundaries;
- deciding where backdrop snapshots are required;
- deciding reflection/light/environment dependencies;
- deciding screen-pass boundaries;
- deciding transient lifetimes and scratch aliasing;
- choosing Present vs Readback terminal work.

In the current implementation these responsibilities are concentrated around:
- `effective_layer_textures()`,
- `sequential_inputs()`,
- the discovery/segmentation portion of `accumulate_sequential()`.

## 4. What remains in Compositor

These are execution primitives and stay:

- `record_pass_chain()`;
- material/shader pipeline compilation and recording;
- `surface_scene_draws()`;
- `stack_over()` and blend kernels;
- `backdrop_pyramid()`;
- reflection/light-cookie GPU generation primitives;
- scratch texture allocation primitives until planner alias slots fully replace them;
- `finalize_into()`;
- `finalize_texture()`;
- `finalize_readback()`;
- texture import/export helpers.

The executor calls these from an already-complete `GpuExecutionPlan`. They must not rediscover semantic dependencies.

## 5. Sink contract

### Present
- takes deferred SceneComposite + camera/window/target;
- records directly to the target;
- never performs synchronous final readback;
- side-effecting and never cacheable.

### Readback
- takes the same deferred SceneComposite;
- renders the same scene semantics;
- adds only the texture/readback terminal operations;
- export/headless use this sink.

No separate preview graph is allowed.

## 6. Removal order

The mechanical cutover order is frozen:

1. SceneComposite + Present/Readback logical roots (implemented).
2. Planner emits an owned composition/run plan instead of compositor discovering runs.
3. Executor operation table maps planned passes to existing compositor primitives.
4. Present path consumes the new plan; verify zero final readback.
5. Readback/export path consumes the same upstream plan.
6. Nested Plate/temporal Scene rendering calls the same executor recursively through a scoped sink, not `prepare_gpu_scene()`.
7. Blocks/overlay special preparation becomes explicit producer operations.
8. Delete CameraProjection GPU wrapper nodes.
9. Delete `NodeKind::GpuScene`.
10. Delete `GpuSceneValue`.
11. Delete `prepare_gpu_scene_with_solver()`, `prepare_gpu_scene()`, and whole-scene culling/work-discovery helpers.
12. Delete legacy compositor planning entry points once no caller remains.

## 7. Owned execution-plan representation

The new composition plan must be owned; it must not borrow `LayerWithPasses` or `SequentialInput<'a>`.

It contains only stable logical resource keys plus immutable per-draw execution metadata required by the primitive recorder.

This prevents `LayerWithPasses` from becoming the new hidden orchestration owner.

## 8. Culling

Whole-scene culling is planner reachability/visibility work.

It must not remain in `plan_frame_graph_scene()` or another renamed whole-scene preparation function.

Conservative visibility decisions may reuse the existing math, but the owner is the planner.

## 9. Cutover acceptance

Before old orchestration deletion:
- Present and Readback plans originate from the same SceneComposite root;
- static content producers are absent from plans after warm-up;
- transform-only edits schedule placement/composition, not content rebuilds;
- temporal snapshots and History remain distinct;
- matte source order does not affect lowering;
- preview sink contains no Readback pass;
- run segmentation is represented in the execution plan, not rediscovered in Compositor.

After deletion:
- repository search finds no production `NodeKind::GpuScene`, `GpuSceneValue`, or `prepare_gpu_scene*`;
- native compile and GPU parity pass;
- representative frame-cost measurements show fewer rebuilt resources/passes and lower frame cost.
