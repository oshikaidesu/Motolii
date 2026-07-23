# M2 Param Pipeline / Element Domain / Constraint Graph disposition (2026-07-16)

Status: **Decision — defer from M2 permanent surface with explicit thaw gates**.

Historical note (2026-07-23): the cutoff blob is dispositioned by [Unit 4N recovery](2026-07-23-historical-param-element-constraint-lineage-recovery.md). The live six-source `DocParam` still matches this decision; PP/ED/CG remain gates, not implemented abstractions. The M2-local task label is written `M2-GAP-15` to avoid collision with global backlog GAP-15 (basic Shape vocabulary).

## Question

The unmerged #176 branch proposes future Param Pipeline, generic Element Domain, and Constraint Graph boundaries. The M2 reclosure gate requires an explicit adoption/defer/reject decision so M3 UI does not silently reinterpret existing `DocParam`, IDs, or evaluation order.

## Known-technology check

- After Effects distinguishes a property's pre-expression and post-expression values. That is an explicit evaluation stage, not merely another source variant ([Adobe expression basics](https://helpx.adobe.com/after-effects/desktop/work-with-expressions/expression-basics/expression-basics.html)).
- Lottie's portable schema represents animation and expression-related data with format-specific structures and constraints; it does not establish one universal modifier pipeline that can be copied into Motolii unchanged ([Lottie schema](https://lottiefiles.github.io/lottie-docs/schema/), [Lottie expressions](https://lottiefiles.github.io/lottie-docs/expressions/)).
- Blender's tracking constraints expose axis/up-axis and ordering choices as explicit constraint semantics. Motolii's existing typed `LookAt`/`Follow` references therefore must not be silently reclassified as arbitrary graph nodes without an ordering/cycle contract ([Blender Track To](https://docs.blender.org/manual/en/latest/animation/constraints/tracking/track_to.html)).

The known systems demonstrate that source, ordered modifiers, constraints, and editable result are separate semantic decisions. They do not justify guessing one permanent abstraction during M2 closure.

## Decision

### 1. M2 closes with the current `DocParam` meaning

The existing closed source variants retain their current interpretation:

```text
Const | Keyframes | Data | Vec2Axes | LookAt | Follow
```

- `LookAt` and `Follow` remain typed references with the existing D3 evaluation order; they are not expression strings or generic constraint nodes.
- Relative Move remains a one-shot D2 macro that writes the selected keys. It is not a persistent post-keyframe offset.
- No `Generator`, `Modifier[]`, generic `ElementId`, or constraint-node collection is added to Document, runtime public API, plugin ABI, journal, or persistent/pseudo-persistent UI state in M2. Transient edit buffers for current values are unaffected.
- Existing fields and variants are not reinterpreted. A future design must be additive (new variant/field plus version gate) or use an explicit D1e migration; it may not change old project meaning in place.

This is a **defer**, not a rejection of the capabilities.

### 2. PP-Gate: Param Pipeline thaw condition

Before any one of the following is specified or implemented, a dedicated PP-Gate decision/spec PR is required:

1. storing a persistent offset after keyed/data-driven values;
2. applying Data and manual correction simultaneously to one parameter;
3. publishing Generator/Modifier as a parameter-plugin kind;
4. showing reorderable evaluation stages in an advanced property UI;
5. standardizing reusable Add/Multiply/Clamp/Remap stages across parameter kinds.

PP-Gate must decide: canonical serialized form; stage order; scalar/Vec2/Color/Bool/Asset type table; inverse editing target (base or result); LookAt/Follow/Data ordering and cycle diagnostics; unknown-plugin preservation versus export refusal; cache invalidation; additive migration; preview/export semantic goldens; and an adversarial review of node-graph/ABI/UI expansion.

After the M2 reclosure gate is separately released, an M3 property panel may inspect and edit the **current single source** before PP-Gate. It must not display a fictional pipeline or persist a second source. This decision does not authorize any M3 product implementation while the 2026-07-15 reclosure stop is in force.

### 3. ED-Gate: generic Element Domain thaw condition

A dedicated ED-Gate is required before any cross-kind generic element identity is persisted or exported through commands/plugins—for example one `ElementId` spanning layers, path points, keyframes, masks, effects, and future 3D components.

ED-Gate must first prove at least three operations that require one cross-kind algebra and decide identity lifetime, ownership, nesting, duplicate/remap, deletion, selection projection, typed rejection, journal/Undo payload, unknown future kinds, and migration. Until then, existing typed IDs remain separate and UI multi-selection is transient/domain-intent composition, not a new Document field.

### 4. CG-Gate: generic Constraint Graph thaw condition

A dedicated CG-Gate is required before adding generic constraint nodes, user-visible constraint ordering, or a constraint plugin ABI. It must decide node/edge types, stage order relative to source/modifier/transform, cycle and singularity policy, multi-target semantics, cache invalidation, unknown-plugin behavior, migration, and semantic goldens.

After the reclosure stop is separately released, existing `LookAt`/`Follow` can be edited in M3 through their typed contract. UI convenience must not turn them into a generic graph or alter their current order.

## M2 reclosure judgment

The three future boundaries do not remain M2 blockers after this decision reaches `main`, because their non-interference boundary is now closed:

1. no current Document/runtime/plugin meaning changes;
2. no M3 UI may persist the deferred concepts before its named gate;
3. future work is additive or explicitly migrated and independently reviewed;
4. the existing D1/D2/D3 rejection and semantic golden tests remain the judge.

## Non-goals

- Selecting the final Param Pipeline representation.
- Creating a universal element or constraint API “for future flexibility”.
- Blocking ordinary M3 property editing of current `DocParam` variants **after** the separate M2 reclosure gate is released.
