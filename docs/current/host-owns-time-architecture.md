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

## One-hour build rule: connect completed systems, do not re-solve them

The one-hour target is literal for the first complete host proof. Feature reduction is not the strategy. Parallel agents are allowed. The target is a host that no longer needs native modification or rebuild for ordinary creation of new work.

After the host is frozen, adding media, SVG/text, materials, WGSL effects, effect ordering, animation/effect parameters and project content must happen outside the native host.

A code agent must not both choose the subsystem boundary and implement its replacement in the same local loop. Before implementing infrastructure, name the mature external owner of that responsibility and state why direct connection is insufficient.

A locally invented test suite is not evidence that replacing a mature subsystem is justified. Such tests only prove the implementation satisfies the cases the same agent chose to test.

### Preselected responsibility owners

These are the default owners to connect unless a concrete fixture demonstrates an incompatibility:

| Responsibility | Default owner | Motolii responsibility |
| --- | --- | --- |
| Video/audio decode and encode | FFmpeg / libavcodec | request input samples at host-owned time; submit finished audio/video for encode |
| Container mux/demux and media protocols | FFmpeg / libavformat | connection and project/export policy only |
| Managed asset reference resolution/publishing | OpenAssetIO | store/pass entity references and resolution context; do not invent an asset database |
| Editorial interchange | OpenTimelineIO | import/export editorial cut information only; do not use OTIO as Motolii's animation evaluator |
| Large image file access/cache and texture lookup | OpenImageIO ImageCache / TextureSystem | connect resolved resources to image/texture consumers |
| HDR scene-linear interchange / multipart image storage | OpenEXR | use the standard representation where an image interchange/cache artifact is actually required |
| Color management / ACES transforms | OpenColorIO | select/configure transforms and connect them to frame processing; do not invent a color pipeline |
| Vector / SVG / text rendering region | ThorVG | connect its native representation/output to the shared GPU frame |
| Path tessellation when geometry is required | lyon | promotion connection only |
| Material description and shader generation | MaterialX / OpenPBR | resource bindings and execution connection; do not invent MotoliiMaterial semantics first |
| GPU execution and shader language | wgpu / WGSL | shared device/resources/frame boundaries |
| Evaluated entity/component state and history/query | Rerun | thin boundary; host still owns temporal evaluation semantics |
| 3D asset interchange | glTF | load/connect native data; do not invent a proprietary exchange format |

OpenAssetIO is specifically a boundary technology: its host design stores entity references where paths would otherwise be stored and resolves them just before use. That is the preferred model for managed assets rather than a new MotoliiAssetManager.

OpenTimelineIO is deliberately narrower: it owns editorial interchange (clips, tracks, transitions, markers, media references), not arbitrary motion-graphics behavior.

OpenImageIO is preferred over a Motolii image cache when the requirement is large image-file access, tile/file-handle caching, texture/environment lookup, or similar established image infrastructure.

OpenColorIO owns color-management transforms/configuration. OpenEXR owns professional scene-linear HDR image interchange. These are not tasks for a custom Motolii color or HDR file system.

MaterialX shader generation is source generation rather than a runtime. Motolii connects generated WGSL and the required bindings to wgpu; this does not justify a new Motolii renderer.

### Infrastructure replacement gate

Before adding a Motolii-owned subsystem such as Encoder, Decoder, AssetManager, ImageCache, ColorManager, TimelineInterchange, MaterialSystem, Renderer, SceneGraph, or equivalent, the implementing agent must answer:

1. What exact responsibility is missing?
2. Which mature project above normally owns it?
3. What concrete fixture fails when that project is connected directly?
4. Can the failure be solved by a bounded adapter/resource contract?
5. Why would a Motolii-owned subsystem remain smaller than that adapter?

If these questions are unanswered, stop implementation and connect the existing owner instead.

### One-hour acceptance condition

The first build is successful only if, after the native host is frozen, a separate agent can create or change a real work without modifying or rebuilding the host:

- add/replace video or audio;
- add/replace SVG/text/image resources;
- add or edit WGSL;
- change MaterialX/OpenPBR material data;
- add/reorder supported effects;
- animate/evaluate through host-owned time;
- seek directly and render deterministically;
- export through FFmpeg;
- save/reopen the work.

If any item requires a host rebuild, report the exact missing connection. Do not hide the missing connection by implementing a reduced substitute.
