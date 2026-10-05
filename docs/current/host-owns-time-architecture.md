# Motolii architecture direction: host-owned time, open visual vocabulary

Status: design direction to preserve before implementation. This records the current conclusion, not a frozen class hierarchy.

## Product test

Motolii is not an After Effects clone. “Two hours to make AE” is an architectural health test, not a delivery deadline: how little Motolii-specific infrastructure must be invented before making an image?

> If it can become an image, it can be put on time.

> Same world does not require the same representation.

Native representations stay native until interaction requires a concrete boundary such as texture, geometry, depth, transform, time, or another GPU resource.

## Do not freeze examples into the core

Interval keyframes, Transform effects, Wiggle, Spring, OpenPBR, ThorVG, and community WGSL are useful vocabulary. They are not automatically Motolii core concepts.

For example, “move while wiggling” can be an animated Transform plus a Transform effect over a range. The temporal model does not have to define overlapping Wiggle segments. Another design may be better later.

Do not add core enums for every known animation, effect, or renderer. Prefer concrete operations first; generalize only when a second real use forces the same contract.

## Host owns time

This is the central invariant:

> The Motolii host owns time. An operation declares which times it needs to produce its output.

Effects must not depend on having been called on the previous playback frame.

Conceptually an operation can declare temporal_requirements(output_time), then evaluate using the temporal inputs supplied by the host.

Examples:

- Blur at t requests t.
- Echo at t requests [t-duration, t].
- Motion blur at t requests shutter samples around t.
- Delay requests t-delay.
- Time remap requests map(t).

The host resolves these requests recursively through dependencies. Example: output@10s -> Echo requests 8..10 -> Transform remaps -0.5s -> Video receives 7.5..9.5.

The same requested time must resolve to the same result whether the user plays forward, jumps directly, scrubs backward, reverses playback, or exports only that frame.

A third-party operation follows the same rule: it declares “to produce t, I need these times.” It does not implement its own scrub correctness.

## Stateful work is still host-managed

Spring, particles, fluid, simulation, feedback and similar operations may need history. That does not grant them ownership of playback time.

An operation declares temporal needs and any required fixed-step/snapshot capabilities. The host owns time resolution, caches, snapshots, invalidation and reconstruction. The exact API is intentionally not frozen here.

## Authoring time is not Rerun time

Rerun is the time-aware entity/component/state/history/query substrate: memory of the evaluated world. It is not required to understand Bezier, Wiggle, Spring, expressions, interval keyframes, or the authoring semantics that produced a value.

Document/authoring vocabulary -> Host temporal resolution -> Evaluated components at t -> Rerun world/history/query.

Keep the Rerun dependency behind a thin Motolii boundary.

## Visual architecture

The center is not a monolithic Motolii renderer. Current direction:

evaluated world -> frame resources -> native contributors (ThorVG / surface work / WGSL or future X) -> FrameGraph/wgpu -> present.

ThorVG is the adopted 2D/vector motion region. lyon is available when a path must become triangle geometry. MaterialX/OpenPBR is a material/shader-generation direction, not a world owner. wgpu/WGSL is the visual commons.

Do not introduce a mother framework merely to make all representations look alike.

## Surface work is bounded, not denied

External review correctly exposed that names such as scene_bindings() can hide real work: lighting, shadows, environment, transparency, visibility and material bindings.

Do not pretend this work disappears, but do not install a second world-owning engine by default. Keep shared facts explicit as frame resources/capabilities where that is the real interaction boundary.

Whether these eventually form one “Surface Specialist” is deliberately not fixed here. That is a useful current hypothesis, not core ontology.

## WGSL is an ecosystem boundary

A major practical bottleneck is obtaining excellent WGSL implementations for visual phenomena. Motolii should not respond by making the core team author every technique.

The core should provide a stable capability contract so official and community operations can declare inputs, outputs, parameters, temporal needs and GPU resource needs. Community shaders/effects should become usable without writing Flutter UI; metadata can expose parameters to the Inspector.

The core owns the grammar; it does not need to own the whole visual vocabulary.

## Current concrete directions

- Flutter for product UI.
- Rerun for evaluated time-aware world/state/history/query, behind a thin boundary.
- ThorVG for native vector/2D motion.
- lyon for path tessellation when geometry is required.
- wgpu/WGSL as the primary GPU commons.
- Existing Motolii FrameGraph as a thin resource/pass scheduler.
- MaterialX/OpenPBR as material/shader-generation direction; exact runtime boundary remains subject to implementation evidence.
- Ray query remains optional/experimental, not a baseline requirement.

## First implementation proof

Do not begin by creating MotoliiAnimationEngine, MotoliiRenderer, a universal operation hierarchy, or a proprietary shader language.

First prove host-owned time with a vertical slice: Video/ThorVG text -> Transform -> Echo -> surface/WGSL as needed -> Bloom -> Stage.

Echo declares its required time range. It must not store the previous playback frame as its correctness mechanism.

Acceptance invariant: play to t, jump directly to t, scrub backward to t, reverse to t, and export only t must resolve to the same image for deterministic inputs.

Then add a genuinely stateful behavior such as Spring. Only then design the minimum snapshot/bake mechanism actually required.

After the temporal proof, use concrete fixtures to expose missing contracts:

1. move while applying a Transform/Wiggle effect;
2. intersecting translucent text layers;
3. glass over video;
4. Glass Garden beauty fixture.

If a fixture exposes one new local contract, add that contract. Return to architecture research only if implementation exposes a responsibility that cannot remain local.

## Guardrails

- Do not inherit AE concepts solely for AE compatibility.
- Do not freeze interval keyframes into the core merely because they are promising UX.
- Do not make effects own playback history.
- Do not make Rerun the animation evaluator.
- Do not hide renderer-sized responsibilities behind names such as scene_bindings().
- Do not introduce a universal renderer interface unless a real fixture forces it.
- Do not require unknown visual technique X to migrate into a Motolii-owned scene representation.
- Do not make experimental ray tracing a baseline dependency.
- Treat community WGSL as part of the extensibility strategy, not an afterthought.

## Review status

External adversarial review moved the architecture from C (unowned large responsibilities) to B: remaining gaps were reducible to local contracts/algorithms rather than a mandatory integrated world-owning renderer.

That is sufficient to proceed to implementation. The next source of truth is the fixture, not another round of framework search.