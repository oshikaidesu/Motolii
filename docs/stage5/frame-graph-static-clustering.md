# FrameGraph static clustering

This is the canonical static-analysis view of the semantic FrameGraph.

The analyzer never evaluates a node and never reads StoreView. It consumes only
the compiled GraphTopology and groups reachable nodes by scheduling/dependency
shape.

## Cluster signature

Each node is classified by:

- semantic domain: Property / Content / Layout / Spatial / Effect / Coverage /
  Analysis / Solver / Scene / View;
- TimeDependency: Static or Exact;
- QualityDependency;
- compiler-declared temporal edge: any InputTime other than Same;
- runtime dependency capability:
  - None
  - CrossNode: may request a different node after ordinary inputs are known;
  - TemporalSample: may request another time of an input/source;
  - Recurrence: may request its own previous-time value.

Nodes with the same signature form one static cluster. Ordinary graph edges are
lifted to cluster-level upstream/downstream edges.

This is intentionally conservative. A node marked TemporalSample or Recurrence
may request no dynamic input for a particular evaluated value. Static analysis
must prefer a harmless false positive to hiding a scheduling boundary.

## API

```rust
let program = SceneProgram::compile(view)?;
let report = program.static_clusters()?;
let markdown = report.to_markdown();
```

The markdown table is intended to become the generated dynamic-node inventory
used during GPU cutover reviews.

## Why this exists

Special cases should first be tested as graph relationships, not implemented as
new renderer branches.

Example: a box-space mask that must be evaluated after solver/block motion is
not automatically a new mask backend. In the static cluster report:

- Mask belongs to Coverage.
- SolverPlan belongs to Solver.
- the report exposes the current cluster edges.

If the desired semantics require Solver -> Coverage and that edge does not
exist, the first question is whether the semantic graph needs a post-solver
Transform/Mask dependency. A renderer-side special producer is the fallback,
not the default answer.

## Dynamic capability map

Current conservative SceneProgram mapping:

| Node kind | Dynamic class |
|---|---|
| AnalysisRequest | CrossNode |
| AnalysisBlob | Recurrence |
| AnalysisOverlay | Recurrence |
| ParticleBirths | Recurrence |
| PlacementSet | TemporalSample |
| MotionMeasure | TemporalSample |
| MotionSamples | TemporalSample |
| TextShape | TemporalSample |
| TextFlow | TemporalSample |
| EffectImages | TemporalSample |
| Particle | TemporalSample |
| CompositeContribution | TemporalSample |

Compiler-declared temporal edges such as TemporalCopy are tracked separately
through InputTime. This distinction matters: an explicit temporal edge is part
of topology identity, while a dynamic temporal sample is selected only after
ordinary input values are known.
