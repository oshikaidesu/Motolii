# FrameGraph node reduction audit

Status: static clustering first pass, PR #511.

## Result

The static-analysis pass identified a complete obsolete scaffold that is not part
of the current SceneProgram production compiler.

Removed in this pass:

- ResolvedWorld
- TextDocuments
- ShapeDocuments
- DocumentCamera
- SharedScene
- CameraProjection
- TextStyle
- ShapeMesh
- Group

The first six belonged to the original InitialTopology / GraphBuilder scaffold.
That scaffold itself was also removed:

- frame_graph/initial.rs
- frame_graph/compiler.rs
- GraphBuilder
- CompilerOutput
- LayerBinding
- InitialTopology and its Camera/Stage wrapper types

The live NodeKind vocabulary is now 40 variants including Custom(u16).

## Reduction policy

A node kind may be removed only when one of the following holds:

1. no current production compiler can construct it; or
2. it belongs exclusively to an obsolete scaffold whose public surface is also
   removed in the same change.

Absence from one compiled document is not enough. The static cluster report calls
those kinds "absent from this topology", not "unused".

## Merge candidates

The static analyzer also reports conservative linear merge candidates. A pair is
reported only when:

- producer and consumer have the same static cluster signature;
- the consumer has exactly one reachable upstream node;
- the producer has exactly one reachable downstream consumer;
- the consumer is not a graph root.

This is a review hint, never an automatic rewrite. A candidate can still encode
an important semantic ownership boundary.

## Next audit

Use SceneProgram::static_clusters() on representative documents and inspect:

- repeated same-signature linear chains;
- Solver -> Coverage gaps;
- recurrence clusters that can be reduced to explicit History ownership;
- temporal-sample clusters that duplicate another temporal edge;
- Scene-domain nodes that only forward values without adding semantics.

The generated markdown table is the canonical input for those decisions.
