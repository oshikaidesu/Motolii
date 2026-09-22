# GPU temporal resources

This document freezes the temporal ownership model used by the GPU execution graph.

## Rule 1: semantic time selection stays on the CPU

`LookbehindProgram` already evaluates the exact source value requested by an effect. GPU execution must not re-run document timing logic or search the timeline.

A `SceneImageSourceValue` is therefore an evaluated request result, not an instruction to seek.

## Rule 2: effect identity survives semantic projection

`SceneLayerValue` carries the `NodeKey` for direct effects, post-placement effects, masks, and the effect owning each image-source row.

The GPU graph may use those identities directly. It must not reconstruct ownership from plugin names, layer order, or vector position after the fact.

## Rule 3: mutable decoder textures are never temporal values

Video/media decoders may overwrite one player texture as time advances. A source at t' becomes an immutable GPU value only after snapshot/copy.

```text
evaluated SceneImageSourceValue
        |
        v
ImageSource logical resource
        |
        +-- Content source -> Copy
        |
        +-- Scene source   -> Composite
        |
        v
immutable resident snapshot
        |
        v
Effect resource dependency
```

## Rule 4: identity and version are separate

Image-source identity is stable for one consumer effect input:

```text
(effect NodeKey, owner LayerId, contribution instance, input index)
```

The version fingerprints the evaluated source value, requested time, namespace, and source contents.

Playback time alone is not an identity.

## Rule 5: named/current and temporal sources have different retention

- namespace 0: persistent residency is allowed; a changed source value replaces the version in the same logical slot.
- non-zero lookbehind namespace: bounded `Temporal` residency.
- feedback recurrence is **not** a lookbehind image source.

Current bounded lookbehind retention is 8 GPU generations. This is a cache policy, not semantic history.

## Rule 6: feedback owns recurrence separately

Persistent effects use `History` resources/checkpoints. Their state is defined by recurrence from the layer in-point and may need checkpoint restore + replay after a scrub.

Do not model feedback as `ImageSource(t-1)`: doing so loses recurrence ownership and makes cache invalidation ambiguous.

## Rule 7: descriptor changes invalidate downstream caches

When a resource version/dependency/lifetime changes, that resource and all resident GPU dependents are dirtied immediately. Downstream resources do not need to fold every ancestor version into their own fingerprint.

## Transitional production bridge

During cutover, `GpuScene` still assembles `LayerWithPasses`, but lowering/materialization now happens inside the `GpuScene` node immediately before assembly. The revision-scoped resident stores are passed directly to the assembler; they are not recovered through `Engine.frame_graph` because that state is temporarily taken during evaluation.

This bridge is temporary. Final C10 removes the whole-scene assembler.
