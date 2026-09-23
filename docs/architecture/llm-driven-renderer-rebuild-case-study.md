# LLM-driven development as an architectural stress test

> **Status: Draft / experiment in progress**
>
> This note records an architectural failure discovered during the renderer rebuild and the hypothesis currently being tested. Final conclusions should be updated after the replacement renderer reaches feature parity and the Glass Garden workload is measured.

## 1. Original premise

Motolii started from a simple premise: build an AE-like motion-graphics tool on top of Rerun / re_renderer, while keeping Motolii's authoring model independent from the renderer.

Motolii owns the semantics of the work: time and timeline, FrameGraph, Group / Repeater / Layout, effects, matte / mask, plates and backdrop relationships, ordered compositing / transmission, feedback / freeze, and JS / Motolii Live.

The renderer should execute those semantics, not own them.

This later became the node/cassette model:

```text
CSS-like authoring
        ↓
Semantic / FrameGraph
        ↓
Cassette: "what does this frame mean?"
        ↓
Lowering
        ↓
Renderer: "how do we draw it?"
```

The intended property is that the semantic layer survives changes in rendering technology.

## 2. How the architecture drifted

During LLM-driven incremental development, individual changes were usually locally reasonable and testable.

A feature needed an intermediate render, so a flush was added. A second view repeated expensive work, so a cache was added. A reflection capture was duplicated, so its key was narrowed. A plate needed preparation during drawing, so preparation leaked into the view path.

None of these changes individually appeared to redefine the architecture. Over time, however, the renderer drifted away from the execution model of Rerun / re_renderer.

Examples discovered during the Glass Garden investigation included:

- `begin_frame` treated as a per-view or per-flush operation instead of an application-frame boundary;
- Stage and Camera draws indirectly owning world preparation;
- world reflection and light resources generated from inside view rendering;
- view order deciding which view paid preparation cost;
- global caches recovering sharing that should have followed from ownership;
- preview views rendering substantially more pixels than their displayed area required;
- frame-scoped staging resources recycled at submission boundaries;
- world and view dependencies mixed inside the Compositor.

The failure was not primarily a set of obviously broken functions. It was a **loss of architectural ownership**.

## 3. Why LLM-driven development amplified the drift

The development loop favored locally safe changes:

```text
observe failure
      ↓
find immediate cause
      ↓
make minimal correction
      ↓
preserve existing tests
      ↓
continue
```

That is useful for ordinary maintenance, but it creates a failure mode for long-running LLM-driven development: existing code, tests, and earlier decisions gradually become implicit axioms.

The agent can become increasingly good at maintaining an architecture that its own previous local decisions accidentally created.

A practical warning follows:

> If several caches, reuse rules, or special cases repeatedly cross the same boundary, stop optimizing the individual cases and re-audit the ownership boundary itself.

## 4. Glass Garden as the stress test

The problem became visible while building `GLASS GARDEN`, a Motolii Live showcase intentionally using expensive realtime features: repeated geometry, WGSL deformation, glass / transmission, reflection, environment lighting, glow, multiple views, live editing, and explicit looping.

The initial workload ran at roughly single-digit frames per second. Profiling exposed genuine inefficiencies, but fixing one duplicated operation repeatedly exposed another ownership-related duplication underneath it.

Eventually the more important question became:

**Why are Stage and Camera responsible for producing these resources at all?**

Audits of Blender, Godot, Filament, Bevy, three.js, and Rerun led back to a clearer boundary:

```text
Document / semantics
        ↓
Evaluation
        ↓
Prepared GPU scene
        ↓
┌───────────────┐
View           View
Stage          Camera
        ↓
Surface
```

Stage and Camera are observations of a prepared frame. They are not owners of the frame.

## 5. The destructive experiment

Rather than continuing to reshape the old orchestration, the current experiment replaces the orchestration itself.

Existing assets remain reusable: FrameGraph, lowering, shape/text rendering, analytic paths, WGSL effects, materials, Group / Repeater, plates, glass/reflection implementation, ordered-transmission semantics, the re_renderer fork, and existing tests.

The old orchestration is retained only as a behavioral and pixel oracle.

The replacement starts from Rerun's actual re_renderer usage, including its `SpatialStage` embedding path:

```text
application tick
        ↓
begin_frame × 1
        ↓
prepare × 1
        ↓
View × N
        ↓
submit
```

Unsupported functionality fails explicitly rather than silently falling back to the old renderer.

At the time of writing, the minimal skeleton and 0/1/2/N-view invariants pass. Basic layers, blend behavior, Blur/Glow, meshes, environment and sky have already matched the old path through pixel-oracle tests. Migration is still in progress.

## 6. The paradoxical test of the cassette model

This failure creates an unusually strong test of Motolii's node/cassette architecture.

The claim was that Motolii's semantic representation should not depend on a particular rendering implementation. That claim is now being tested under adverse conditions.

The renderer orchestration became sufficiently problematic that replacing it became preferable to continuing incremental repair. Yet the replacement does not require rebuilding Motolii's authoring semantics.

```text
                         ┌→ old renderer orchestration
Motolii semantic cassette
                         └→ new Rerun-aligned orchestration
```

If the replacement reaches feature parity while preserving document semantics and expected pixel output, that is stronger evidence than an architectural diagram could provide.

**The renderer is replaceable not because it was designed to look replaceable, but because it was actually replaced.**

## 7. Ownership should follow dependencies

The rebuild suggests a more general rule: whether a computation belongs to the prepared frame or to a View should follow from its inputs, not its feature name.

Inputs such as document geometry, time, and environment can be evaluated at frame/prepared-scene scope. Inputs such as the current view image, projection, or viewport history necessarily belong to View scope.

This makes world reflection naturally shareable while ordered transmission, which reads the already-rendered view image, remains view-owned.

Longer term, explicit graph dependencies may allow this ownership to be derived mechanically rather than assigned feature by feature.

## 8. New invariants for LLM-driven development

Architectural principles should become executable invariants. Current candidates include:

- one renderer `begin_frame` per application tick;
- at most one semantic/world preparation per document frame;
- increasing View count must not multiply world preparation;
- world-owned captures must not scale with View count;
- View rendering must not mutate the prepared world;
- shared mesh/material uploads must not duplicate merely because another View exists;
- frame-scoped resources must not retire at View or intermediate-submission boundaries;
- unsupported features must fail explicitly rather than silently enter an older execution path.

There is also a project-level rule:

**Rerun-based means architectural dependence, not merely a crate dependency.**

Motolii owns motion-graphics semantics. Generic renderer infrastructure—frame lifecycle, View lifecycle, GPU resource lifetime, staging, pooling, submission, generic batching and instancing—should follow Rerun / re_renderer unless a Motolii-specific semantic requirement explains the divergence.

## 9. What remains to prove

This draft intentionally does not claim that the experiment has succeeded.

The replacement still needs to demonstrate:

- feature parity for the required Motolii Live path;
- pixel equivalence where semantics are unchanged;
- correct 0 / 1 / 2 / N View behavior;
- a single Rerun-aligned frame lifecycle;
- correct ownership for plates, transmission, feedback, reflection and related operations;
- removal of obsolete orchestration and compatibility fallbacks;
- successful execution of Glass Garden;
- realtime performance measurement after architectural duplication is removed.

Only then can the remaining Glass Garden cost be interpreted as the actual cost of the intended rendering workload.

## 10. Preliminary conclusion

The current evidence points to two lessons.

First, long-running LLM-driven development can produce architectural drift even when individual changes are reasonable, reviewed, and tested. Local correctness is not sufficient to preserve global ownership.

Second, that same failure is becoming a destructive test of Motolii's core architectural claim.

The lower execution layer drifted far enough that replacing it became attractive. The semantic layer did not need to be replaced with it.

If the migration completes successfully, the failure will provide a paradoxical form of validation:

**LLM-driven local optimization damaged one implementation of the renderer, while the ability to discard that implementation demonstrated that Motolii's semantic architecture was genuinely independent from it.**

That is not a reason to accept architectural drift. It is a reason to turn the boundary that survived it into a tested invariant.
