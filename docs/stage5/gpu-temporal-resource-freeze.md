# GPU temporal/resource architecture freeze

This document freezes the control-plane contract for the remainder of the GPU rewrite.
After this point the migration is intended to be mechanical: move one producer at a time,
then delete its legacy owner.

## 1. One clock owner

Semantic evaluation owns time selection.

`LookbehindProgram` already resolves effect image requests into concrete
`SceneImageSourceValue` values at exact times. GPU lowering MUST NOT recompute effect
offsets, seek rules, group membership, or layer selection.

The GPU receives evaluated source requests and only owns residency, immutable snapshots,
history state, scheduling, and execution.

## 2. Effect identity is semantic identity

`SceneLayerValue.effect_keys` and `after_effect_keys` preserve the `EffectProgram`
`NodeKey` beside each evaluated `ResolvedEffect`.

Every GPU Effect resource is anchored to that key. No GPU code may recover identity from
plugin id, vector index, layer id, current frame, or debug formatting.

Group/Whole propagation must carry those keys with the values.

## 3. Image input model

Each translated effect pass owns zero or more image-source slots.

For every slot GPU lowering creates:

```
source semantic value / source scene
            |
            v
      ImageSource
            |
            v
        Snapshot
            |
            v
          Effect
```

- `ImageSource` is the logical dependency requested by the evaluated effect.
- `Snapshot` is the immutable GPU-visible image consumed by the effect.
- Effects NEVER read a mutable decoder/player texture directly when an auxiliary image is
  requested.
- A scene-valued source is rendered to a composite texture first, then becomes a Snapshot.
- A content-valued source is materialized using the ordinary content backend, then copied
  to a Snapshot when the backing texture can be overwritten.

Identity for ImageSource/Snapshot is anchored to the owning effect NodeKey plus stable
(pass-slot, image-slot). Version is derived from the evaluated source value, exact source
time and namespace. Current comp time or evaluation generation alone is not a version.

## 4. Lifetimes

| Resource | Lifetime | Rule |
| --- | --- | --- |
| static/evaluated Content | Persistent | retained while value-version matches |
| Placement | Persistent | independent from Content |
| Effect output | Persistent when cacheable | invalidated by true inputs only |
| same-frame named ImageSource | Frame unless safely backed by immutable resident content | no accidental long-lived alias to decoder textures |
| temporal Snapshot | Temporal | retain a small generation window; regenerate from semantic value after eviction |
| feedback state | History | explicit recurrence owner; never treated as an ordinary temporal snapshot |
| Matte/Plate/Flatten output | Persistent when inputs match | each has its own producer |
| Scratch | Frame | aliasable by planner liveness |
| Present/Readback | Frame | sink only |

`History` and `Temporal` are intentionally different:
- Temporal = an already-defined value at another time.
- History = state whose value is defined recursively from previous execution.

## 5. Feedback ownership

Existing `FeedbackKey { layer, copy, chain, index, screen, namespace }` maps to a History
resource identity. The concrete backend may initially reuse the compositor's proven
checkpoint textures, but ownership moves to the GPU execution layer.

Rules:
1. normal frame advancement mutates only the matching History resource;
2. seek restores a checkpoint or replays from the semantic in-point;
3. temporal lookbehind namespaces never mutate production namespace 0;
4. document/value invalidation dirties only affected histories;
5. History is not evicted by ordinary temporal-resource retirement.

## 6. Matte, Plate and Flatten producers

These are explicit producers, never hidden work discovered by the compositor.

```
Content -> Effect -> Mask -----> contribution image
                         \-----> Flatten
contribution + matte source ---> Matte
plate members -----------------> Plate
```

A matte source depends on the source contribution's final pre-matte image. The lowerer
must therefore be order-independent: declare contribution output identities for the whole
scene first, then connect producers/dependencies in a second phase.

No pass may require its matte/source resource to have been lowered earlier in scene order.

## 7. Scene root and sinks

The planner owns the final composition root:

```
contribution outputs
        |
        v
 SceneComposite
    /       \
Present   Readback
```

Preview, export and headless share the same upstream graph. They differ only at the sink.
Normal preview has zero synchronous Readback passes.

## 8. Concrete executor boundary

The executor receives a finished `GpuExecutionPlan`.

It may:
- obtain/create concrete textures, buffers, models and pipelines;
- reuse resident payloads;
- allocate physical scratch slots selected by the planner;
- record and submit commands;
- update residency/history after successful execution.

It may NOT:
- inspect the document;
- rediscover effect/image/matte dependencies;
- walk a whole Scene looking for work;
- decide which semantic time to evaluate;
- silently create a second planning graph.

## 9. Mechanical migration order

1. preserve Effect NodeKeys through every semantic scene/group path;
2. lower ImageSource + Snapshot edges from `SceneImageSourceValue`;
3. move `frame_graph_image_sources()` snapshot work to those producers, then delete it;
4. move feedback storage/checkpoint ownership to History resources;
5. move Mask output producer;
6. move Flatten producer;
7. two-phase declaration + Matte producer;
8. Plate producer;
9. add SceneComposite root + Present/Readback sinks;
10. make compositor execute the plan instead of discovering work;
11. remove `GpuSceneValue`, `NodeKind::GpuScene`, `prepare_gpu_scene_with_solver()`,
    `prepare_gpu_scene()`, and their whole-scene planning helpers.

## 10. Cutover gates

The rewrite is not complete until all are true:

- transform-only edits do not rebuild Content;
- static overlays survive video frame advancement;
- effect source dependencies are explicit graph edges;
- matte lowering is independent of scene traversal order;
- feedback and temporal lookbehind have different resource ownership;
- unchanged GPU resources remain resident across frames;
- planner liveness drives transient aliasing;
- preview/export/headless share one upstream execution graph;
- preview performs no synchronous final readback;
- compositor contains no semantic dependency discovery;
- old GpuScene orchestration is deleted, not retained as fallback;
- native compile/parity tests pass;
- representative native measurements show reduced rebuilt resources/passes and improved frame cost.
