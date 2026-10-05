# Motolii 30-minute core freeze

Status: **current architecture direction, frozen before implementation**
Date: 2026-10-06

This document records why Motolii is leaving the scratch-built application model. It is intentionally written before resolving the remaining runtime questions. Do not silently fill those gaps by inventing Motolii subsystems.

## Core thesis

**Motolii is a Skin.**

Motolii's value is not owning another renderer, scene graph, animation engine, material system, media engine, audio engine, compositor, cache architecture, or GPU scheduler. Its job is to expose mature technologies as one coherent MV-making experience while preserving their native document models.

The goal of the 30-minute constraint is not to implement an AE clone in 30 minutes. It is an architecture lint: if a capability requires Motolii to fork, repair, or recreate a mature subsystem, ownership has leaked back into Motolii.

## Why this changed

The old path repeatedly turned a production need into a Motolii subsystem:

- animation -> property/evaluation machinery
- GPU dependencies -> FrameGraph / execution machinery
- 3D -> custom renderer work
- temporal effects -> history/cache machinery
- text -> text pipeline
- media -> media pipeline
- audio -> audio pipeline

Each local decision was defensible. Together they reconstructed the organs of After Effects.

The FrameGraph made the failure mode explicit. Resource lifetimes, barriers, cache, history, pass ordering and scheduling are real problems, but a completed renderer/runtime should own them. Renaming the same ownership to RenderPlan, ExecutionGraph, VisualWorld, etc. is not a fix.

Renderer research exposed the same problem at another level: assembling GI, reflections, shadows, transmission and post from separate parts leaves the integration responsibility in Motolii. Kajiya and other integrated renderers were evidence that visual quality comes from coupled renderer decisions, not a bag of algorithms.

A second failure followed the first correction. “Do not implement; search OSS” became component-category search: animation system -> 3D renderer -> compositor -> another renderer. This produced loops. The unit of search must instead be an indivisible production responsibility, starting from the artwork that must exist.

EffectCraft then provided a useful control specimen: a clean-room AE implementation exposes the organs required when an AE-like application owns everything itself. Motolii uses it as an anatomy repository, not as an architecture to copy. For every organ found there, ask why it exists and which mature technology can own that responsibility instead.

Lumit, OpenRV, Natron and OpenFX provided further evidence that effects, mattes, multi-frame dependencies, caching and community shaders are already named and solved responsibilities. Do not create Motolii equivalents merely because the current repo lacks them.

## Authoritative world

Rerun was chosen for a specific reason:

**heterogeneous document models must be composable without first becoming Motolii document models.**

The desired shape is:

```text
Rerun world
├─ SVG / ThorVG document
├─ MaterialX / OpenPBR document
├─ video/media
├─ mesh/glTF
├─ Tracktion Edit
├─ text document
└─ camera
```

Rerun's common responsibility should stay close to:

- identity
- time-aware state/history/query
- transform
- relationship

Do not normalize these into `MotoliiShape`, `MotoliiClip`, `MotoliiMaterial`, `MotoliiAudioTrack`, or another unified scene/document hierarchy.

Current Rerun direction is the data-side crates (`re_chunk_store`, `re_entity_db`, `re_query`, `re_tf`) behind a thin boundary. Rerun Viewer is not the Motolii shell. The existing `re_renderer` fork is not part of this frozen core direction.

## Projection, not conversion

“Documents can coexist” is too weak. Visual documents must be able to become physical/visual objects in the same world when the artwork requires it.

Examples:

- video -> decoded image -> material/surface projection
- SVG -> ThorVG raster surface, or temporary tessellated geometry when physical depth is required
- MaterialX -> renderer-native material projection

The original document remains authoritative. The geometry/surface/material used for rendering is a projection, not a replacement document.

The world test therefore includes depth/occlusion and, where the selected runtime supports them, reflection, refraction, lighting, shadowing, DOF/motion/post across heterogeneous sources.

## Responsibility owners already selected

These are not invitations to recreate equivalent Motolii layers.

- **Rerun** — time-aware world memory/query/relationships.
- **MaterialX + OpenPBR** — authoritative material/look document.
- **ThorVG** — vector/SVG rendering; use GPU integration rather than inventing a vector renderer.
- **Tracktion Engine** — authoritative audio edit/playback/sequencing responsibility. Do not continue the custom Motolii audio engine.
- **GStreamer** — realtime media transport/clock/A-V synchronization candidate family; use its clock/timestamp/segment semantics rather than making Rerun the playback clock.
- **FFmpeg** — codec/mux/export responsibility; not the interactive composition document.
- **OCIO / ACES / libplacebo** — existing color/HDR/video-finishing territory; do not equate RGBA16F with a color pipeline.
- **wgpu-graft / native external-image mechanisms** — known route for GPU-image interop when applicable; do not invent a Motolii interop framework first.

## Existing designs used as evidence, not automatically as dependencies

- **EffectCraft** — AE anatomy/control specimen.
- **Lumit** — evidence for a WGSL effect asset contract, parameters, mattes, blends, temporal/effect graph responsibilities.
- **OpenRV** — evidence for image-processing graphs, dependency/cache, multi-input/multi-frame shader processing.
- **Natron / OpenFX** — evidence for ROI and temporal frame-dependency contracts such as “frames needed”.
- **Hydra 2 Scene Index** — evidence for evaluated visual projection, filtering, merging and dirty propagation. Whether it belongs in the actual 30-minute path remains unresolved.
- **Aurora / Filament / Falcor / other completed renderers** — renderer candidates/evidence only until the remaining visual-runtime question is resolved.

Do not turn an evidence source into the new Motolii foundation by default.

## Community effect direction

WGSL matters as a potential **community effect asset contract**, not as a reason to preserve the old renderer.

Desired author experience:

```text
effects/foo.wgsl
+ metadata / declared inputs / parameters / frame dependencies
-> hot reload
-> parameters exposed to the Skin
-> animation without native host rebuild
```

Lumit/OpenRV/OpenFX are evidence for the shape of this responsibility. Do not respond by building a Motolii EffectGraph, TemporalGraph or shader translation framework.

## Old Motolii ownership is not the new foundation

The new path must not depend on preserving:

- `motolii-render` as the product renderer
- custom FrameGraph / GPU execution machinery
- custom compositor
- the `re_renderer` fork
- custom audio/media/export engines
- custom text rendering
- custom property/animation evaluation as the universal document model
- physics/particle/ghost/lookbehind programs as core infrastructure

Do not benchmark or rescue these in order to justify carrying them forward. They may remain in the repository while the new path is established, but the new core must not be designed around them.

`motolii-glue` is an expedient entry point, not a blessed architecture. Flutter remains the Skin direction; existing assets may remain useful independently of the old runtime.

## Anti-reinvention rule

When a gap appears, do **not** create a Motolii subsystem to bridge it.

Especially forbidden as reflexive solutions:

- Motolii SceneGraph / VisualWorld
- Motolii Material model
- Motolii renderer
- Motolii FrameGraph / RenderPlan / ExecutionGraph
- Motolii EffectGraph / TemporalEvaluationGraph
- Motolii MediaEngine / AudioEngine
- unified Motolii Document model
- generic `IRenderer`, `IBackend`, or large replaceability framework

Changing the noun does not change the ownership.

If an integration requires a large adapter, upstream fork, renderer surgery, duplicate authoritative representation, or a new subsystem, treat that as evidence that the combination is wrong before writing the subsystem.

## Artwork test

Do not validate the core with a cube or a single isolated shader.

The target is one MV scene containing, as early as possible:

- live-action video
- SVG/2D character
- 3D geometry
- camera
- light/emissive content
- thick coloured transparent glass
- external/community effect
- temporal effect
- audio

It must ultimately support direct seek, reverse/scrub, deterministic frame evaluation and export.

The point is to test a real heterogeneous artwork, not prove that individual APIs can be called.

## Intentionally unresolved after this freeze

Do **not** solve these by inventing local infrastructure. They are the next investigation/implementation decisions:

1. The completed realtime visual runtime/renderer that can own the coupled beauty-rendering responsibility while accepting the required projections.
2. The concrete effect/compositing runtime path for masks, mattes, adjustment/effect stacks, temporal/history processing and the WGSL community contract.
3. Whether Hydra 2 Scene Index is actually needed in the execution path or is only a useful model/evidence source.
4. The exact playback-clock boundary between GStreamer/media time and Tracktion audio, while keeping Rerun as queried world state rather than the master clock.
5. The thinnest renderer-specific projection boundary; do not pre-emptively solve this with a generic renderer abstraction.

These unresolved items are deliberate. Resolve them explicitly in follow-up work and update this document/decision record. Do not let an implementation silently decide them.

## 30-minute rule

The 30-minute core experiment is allowed to connect mature systems and produce artwork. It is not allowed to make missing systems.

A structural need for a large adapter, fork, custom renderer feature, custom graph, custom cache/history architecture or duplicate document model is a rejection signal, not an implementation backlog.

**The objective is not to implement Motolii faster. The objective is to remove what Motolii needs to implement.**
