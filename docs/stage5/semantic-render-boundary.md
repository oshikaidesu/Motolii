# Semantic graph and render lowering boundary

## Decision

Motolii has one authored/evaluated **semantic graph**. CPU and GPU are not separate
semantic node families.

`NodeKind`, `SceneProgram`, and the existing FrameGraph programs describe Motolii
meaning: properties, layout, transforms, text, masks, effects, relations, scene
composition, and other evaluated document concepts. A node must not gain a CPU/GPU
identity merely because one backend executes it.

Rendering consumes evaluated semantic values through a separate lowering boundary:

```text
Document
  -> Semantic FrameGraph
     Property / Layout / Transform / Text / Mask / Effect / Relation / SceneComposite
  -> Render / Compute lowering
     Raster / Compute / Filter / Composite / Transfer
  -> Resource planning
     identity / dependencies / residency / liveness / temporal history
  -> backend
     wgpu
     CPU/reference where useful
```

The render-work vocabulary is intentionally small. It is an execution IR, not a
second copy of Motolii semantics. The five work families above are an initial
classification, not a requirement to force every operation into exactly five
variants.

## Consequences

1. Do not add `*CpuNode` / `*GpuNode` pairs for a semantic operation.
2. A semantic feature is defined once. Backend acceleration is supplied by lowering
   its evaluated values/resources to render/compute work.
3. GPU residency, transient aliasing, temporal history, uploads, copies and readback
   belong below the semantic graph.
4. CPU/reference execution may consume the same lowered IR where practical; parity
   should not require two independent definitions of Motolii meaning.
5. Backend specialization is allowed when measured or required, but it must not
   become a new semantic owner.
6. Existing production behavior remains the migration oracle unless an explicit
   decision overrides it. Missing prose alone is not a reason to reopen semantics.

## Current code reading

Most existing FrameGraph `NodeKind` variants already follow this rule: `Transform`,
`WorldTransform`, `Layout`, `Mask`, `Effect`, `SceneComposite`, etc. They are
semantic roles rather than execution-device roles.

`GpuScene` is the obvious boundary leak. It should not be mechanically deleted:
first move its resource-preparation responsibility behind render lowering, preserving
the dependency/residency/liveness/history work already built around it. Once no
semantic consumer depends on GPU-specific identity, remove the semantic
`NodeKind::GpuScene`.

## Migration rule

This decision does **not** pause the current all-node cutover.

Continue migrating legacy evaluators into the semantic FrameGraph. In particular,
the remaining transform evaluator should converge on `TransformProgram`; it should
not be replaced by a second GPU transform semantic implementation.

After semantic ownership is singular:

1. classify current GPU graph operations as semantic meaning vs execution work;
2. retain semantic meaning above the boundary;
3. lower execution work into the small render/compute IR;
4. move resource planning below that boundary;
5. delete GPU-specific semantic identities once their callers are gone.

Stop for human input only when implementation requires choosing between two or more
externally observable semantics. Refactoring execution placement is not such a
choice.

## Completion gates

The boundary is complete when:

- every authored/evaluated meaning has one semantic owner;
- no semantic node identity is named for CPU/GPU placement;
- render work depends on evaluated semantic values/resources, not legacy whole-scene
  evaluators;
- resource lifetime/residency/history are owned below semantic evaluation;
- production references to legacy semantic owners are zero, except explicit
  adapters/oracles allowed by the migration manifest.
