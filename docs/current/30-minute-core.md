# Motolii 30-minute core freeze

Status: **current architecture direction, frozen before implementation**
Date: 2026-10-06

This document records why Motolii is leaving the scratch-built application model and freezes the responsibility boundaries that have now been resolved. Do not reopen a settled responsibility by inventing a Motolii subsystem.

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
- **Slang** — portable shader generation/execution target and differentiable computation. MaterialX 1.39.5+ Slang generation is part of the selected path.
- **Slang autodiff + existing local-AD patterns (including Falcor's material/scene-gradient work)** — selected extension path for parameter↔image gradients. Motolii does not own an AD engine or inverse renderer; expose only the gradient paths the artwork needs.
- **ThorVG** — vector/SVG rendering; use GPU integration rather than inventing a vector renderer.
- **Tracktion Engine** — authoritative audio edit/playback/sequencing responsibility. Do not continue the custom Motolii audio engine.
- **GStreamer** — realtime media transport/clock/A-V synchronization owner; use its clock/timestamp/segment semantics rather than making Rerun the playback clock.
- **FFmpeg** — codec/mux/export responsibility; not the interactive composition document.
- **OCIO / ACES / libplacebo** — existing color/HDR/video-finishing territory; do not equate RGBA16F with a color pipeline.
- **wgpu-graft / native external-image mechanisms** — known route for GPU-image interop when applicable; do not invent a Motolii interop framework first.

## Existing designs used as evidence, not automatically as dependencies

- **EffectCraft** — AE anatomy/control specimen.
- **Lumit** — evidence for a WGSL effect asset contract, parameters, mattes, blends, temporal/effect graph responsibilities.
- **OpenRV** — evidence for image-processing graphs, dependency/cache, multi-input/multi-frame shader processing.
- **Natron / OpenFX** — evidence for ROI and temporal frame-dependency contracts such as “frames needed”.
- **Hydra 2 Scene Index** — evidence for evaluated visual projection, filtering, merging and dirty propagation; use locally only when a selected projection/renderer benefits from it. It is not a required Motolii core layer.
- **Aurora / Filament / Falcor / other completed renderers** — local projection/rendering capability sources, not a canonical Motolii renderer. Falcor is additionally evidence/source for existing local differentiable-rendering paths.

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

## World <-> Image is a round trip

Physical projection is not the terminal stage. Motolii must allow an authoritative document to project into a world object or an image, allow a world render to become an image-processing input, and allow an image result to be projected back into the world when the artwork requires it.

```text
authoritative documents / Rerun @ t
        |                     |
        v                     v
   physical world         image source
        |                     |
        +------> IMAGE <------+ 
                  |
        WGSL / OpenFX / matte / merge /
        retime / multi-frame / adjustment
                  |
             image result
              /       \
          output    world input
                   (texture/emission/etc.)
```

This is an artistic composition relationship, not permission to recreate the old Motolii GPU FrameGraph. Resource scheduling, barriers, residency and renderer-internal execution remain the responsibility of the selected mature runtimes.

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

## No architecture question marks remain in this freeze

The previously listed renderer/effect/Hydra/clock/projection questions are no longer architecture vacancies that Motolii may fill with new subsystems.

- There is no canonical Motolii renderer. Use completed renderers or local rendering solutions at projection boundaries according to the artwork.
- Effects/compositing are exposed through existing contracts and local solutions (OpenFX semantics, WGSL community effects, mature image/video processing runtimes); do not create a Motolii EffectGraph/TemporalGraph runtime.
- Hydra 2 is optional local evaluated-scene infrastructure, not a mandatory core layer.
- Media time is owned by mature media/audio timing systems; Rerun is queried world state, not the master playback clock.
- Renderer-specific projection stays local and concrete; no generic Motolii renderer abstraction is required.
- Differentiable/inverse operations use Slang autodiff and existing local derivative patterns only where gradients are needed; the rest of the renderer need not become differentiable.

A future implementation may discover a concrete incompatibility. That is a technology-selection/replacement event, not an invitation to add a new Motolii-owned architecture layer.

## 30-minute rule

The first 30 minutes are the baseline architecture lint: connect mature systems and produce artwork, but do not make missing systems. The budget may extend to 60 minutes when a currently verifiable extension materially expands the creative model (for example parameter -> image becoming parameter <-> image through existing Slang autodiff/local-AD paths). The extra time may only expose capabilities that already exist in the selected technologies; it may not fund a new Motolii subsystem.

A structural need for a large adapter, fork, custom renderer feature, custom graph, custom cache/history architecture or duplicate document model is a rejection signal, not an implementation backlog.

**The objective is not to implement Motolii faster. The objective is to remove what Motolii needs to implement.**
