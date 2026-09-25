# Group Relations: direct relationship editing proposal

> Status: proposal / research input. This PR does not freeze schema or UI.
>
> Axis: **minimum action, maximum relationship**.

## Problem

A Group currently answers **which objects belong together** and gives them a shared transform.
That is useful, but it does not answer the next production question: **how should those members relate to one another, to time, or to another object?**

The common failure mode is to expose the implementation graph itself:

- set 300 positions individually;
- build a Geometry Nodes-style graph;
- write an expression;
- open a separate graph editor just to describe a common production intent.

The user usually still thinks they are editing **one object**: a jewel field, a ring, a fence, a pipe, a building, a text cloud.

## Working hypothesis

Keep Group as the ordinary container, and allow a small set of **typed relations** to be attached to it.

```text
Group
  members: Jewel × 300
  relations:
    Arrange: Circle
    Face: Center
    Timing: Stagger Along
```

A relation owns a rule, not 300 authored child values. Members remain ordinary objects and the relation is evaluated from the same document/time model.

This is deliberately not a node editor.

## Direct interaction

The normal interaction should follow a well-established production pattern:

1. **Select the subject** — e.g. `Jewel Group`.
2. **Choose one verb** — e.g. `Along Path`, `Face Target`, `Scatter On`.
3. **Pick the target on Stage if the verb needs one** — e.g. a Path, Camera, Surface, or Object.
4. The relation appears on the selected Group and remains inspectable/removable/animatable.

Examples:

```text
Jewel Group → Along Path → select Path
Jewel Group → Face Target → select Glass Sphere
Jewel Group → Scatter On → select Surface
Profile + Path → Sweep
```

The Stage remains the primary place for target selection. Do not require users to wire ports for these common intents.

## Prior-art convergence

This proposal follows existing high-level interaction rather than inventing a new graph UI:

- **CSS / Figma Auto Layout**: the parent declares a relationship and child placement follows it.
- **Cinema 4D MoGraph**: Cloner + Effector + Fields turns one object into a governed collection; selecting a Cloner and adding an Effector establishes the relationship.
- **Houdini shelf tools / SOPs**: common procedural graphs are collapsed into high-level actions such as Copy to Points, Scatter and Align, and Sweep; source/target selection establishes the graph.
- **Rive constraints**: Follow Path, IK, Distance, Rotation, Transform and related constraints are attached to ordinary hierarchy objects rather than requiring a separate node program.
- **Cavalry Duplicator**: distribution, per-copy variation and time offset are presented as properties of a generated collection.

The reusable lesson is not any one product's UI. It is:

> **subject → verb → optional target**

for common typed relationships.

## Candidate relation vocabulary

This list is intentionally provisional. It is a research inventory, not a schema commitment.

| Intent | Examples | Broad effect |
|---|---|---|
| Repeat | count / copies | one → many |
| Arrange | Grid / Stack / Circle | member position |
| Along | Path / Surface / Volume | map members onto structure |
| Face | Camera / Center / Target | member orientation |
| Follow | target + offset | maintain typed dependency |
| Scatter | region / density | distribution |
| Sweep | Profile × Path | generate geometry from relation |
| Influence | Field / falloff | vary an existing relation spatially |

Random, Noise, Falloff and Stagger should first be tested as **variation/modulation of a relation**, rather than promoted to unrelated top-level systems.

## Why Group is the starting point

Do not replace Group with a new universal object yet.

```text
Group
  + no relation      = ordinary Group
  + Arrange          = layout-bearing Group
  + Along / Scatter  = distribution-bearing Group
  + Face / Follow    = constrained Group
  + Influence        = spatially modulated Group
```

This preserves the current mental model and lets evidence determine whether a broader Assembly/Collection abstraction is actually necessary.

Relations may eventually need to target members across multiple groups. That is a later boundary question; it is not a reason to prematurely replace Group.

## Time model

A relation must participate in Motolii's existing `f(t)` evaluation rather than introduce frame-to-frame simulation.

Examples:

- Circle radius changes with time.
- Sweep progress grows continuously.
- Stagger maps member/index/position to time offset.
- A target moves and `Face Target` reevaluates at the requested time.
- A Field moves through a collection and changes relation strength.

The desired distinction from a procedural modeling graph is that **generation and animation are not separate phases**.

## Product criterion

Evaluate each proposed relation by the axis:

> **How many authored values can one meaningful action replace while keeping the result understandable and editable?**

A relation is valuable when one action establishes a large, persistent relationship:

- `Scale = 120%`: roughly one value.
- Group transform: one action affects N members identically.
- `Arrange Circle`: one action determines N positions.
- `Face Target`: one action determines N orientations over time.
- `Stagger Along Path`: one action determines N time offsets.
- `Sweep`: one relationship replaces an entire generated surface.

Feature count is not the objective.

## UI guardrails

For the first prototype:

- no Galaxy/graph view;
- no exposed node graph;
- no string expressions;
- no new hidden Null/controller objects;
- no second ownership model for child transforms;
- no automatic inference of user intent when several relations are plausible.

Prefer a conventional command plus Stage target picker. Advanced UI may inspect the typed relation after it exists, but Direct and Advanced must edit the same meaning.

## First validation fixtures

Before freezing schema, test whether a small vocabulary can reproduce familiar high-value procedural outcomes:

1. 300 Jewel ring / field.
2. Fence along a path.
3. Pipe/ribbon from profile + path.
4. Stair stack.
5. Building floors/windows.
6. Text/glyphs along a path.
7. Objects facing a moving target.
8. Field-driven scale/time wave through a collection.

For each fixture record:

- number of user actions;
- number of individually authored child values;
- whether relation survives member count changes;
- whether it remains editable at arbitrary time `t`;
- whether the same primitive is reused by another fixture.

## Stop condition

Do **not** implement a broad public schema from this proposal alone.

First search and prototype the relation vocabulary against prior art and the fixtures above. Stop for product judgment if:

- the same fixture requires mutually incompatible relation meanings;
- Group cannot own the relation without confusing ownership;
- cross-group relations force a new first-class seat;
- a relation needs sequential simulation rather than pure evaluation at `t`.

The goal of this PR is to preserve the design direction:

**Do not make users author a graph when one meaningful relationship can own the graph for them.**
