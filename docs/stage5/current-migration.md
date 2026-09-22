# Current migration state

This file is the main-branch handoff for active architecture migrations. It is intentionally short.

## Active target

Separate Motolii's semantic FrameGraph from render/backend execution.

Semantic meaning is owned once by the FrameGraph. Render execution is lowered into a backend-neutral cassette IR and executed by interchangeable backends.

## Active implementation PRs

- #526 — semantic/render ownership decision
- #527 — cassette bootstrap beside GpuScene

## Current implementation state

- Semantic FrameGraph already owns document meaning such as Property, Layout, Transform, Text, Mask, Effect, Relation and SceneComposite.
- A backend-neutral RenderGraph now exists with Raster / Compute / Filter / Composite / Transfer work families.
- The headless product pixel path has been routed through the cassette composite path.
- Plain Text / Shape / Media / Material / Particles can be prepared without GpuScene when no unsupported effects/matte/plate semantics are present.
- GpuScene remains the reference/oracle for unsupported paths until parity is established.
- The real-pixel CI proof exists but still depends on Linux native build/runtime packages; CI environment failures are not migration-semantic failures.

## Completion gates

The migration is complete when all are true:

1. Every authored/evaluated meaning has one semantic owner.
2. Semantic node identity does not encode CPU/GPU placement.
3. RenderGraph work is backend-neutral and does not own Motolii semantics.
4. Resource identity/dependencies/residency/liveness/history live below semantic evaluation.
5. Production rendering no longer requires NodeKind::GpuScene.
6. Legacy semantic owners have zero production references except explicitly allowed adapters/oracles.
7. A real-pixel cassette path passes in CI.
8. Static migration inventory passes its production-owner gates.

## Resume protocol

When resuming after an interruption:

1. Read this file.
2. Inspect the active PR heads and CI checks.
3. Continue the smallest unmet completion gate.
4. Do not reopen settled semantics merely because prose is incomplete.
5. Stop for human input only when code requires choosing between multiple externally observable semantics.
