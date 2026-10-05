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


## Rerun reuse audit: make it the data backplane, not the behavior engine

A fresh audit of current Rerun architecture shows that Motolii can reuse substantially more than only a basic ChunkStore.

Prefer the existing Rerun crates directly, behind one thin version-sensitive boundary:

| Motolii need | Rerun owner/capability | Decision |
| --- | --- | --- |
| Entity identity/path | re_log_types / EntityPath | reuse |
| Component-oriented world data | re_types_core / re_sdk_types + custom Arrow components | reuse |
| Time-indexed component storage | re_chunk / re_chunk_store | reuse |
| Entity-oriented in-memory database | re_entity_db | reuse |
| Latest-at and range queries | re_query / storage-engine query cache | reuse |
| Higher-level dataframe queries | re_dataframe / re_datafusion | reuse where useful |
| Spatial transform processing | re_tf + Transform3D semantics | reuse before inventing Motolii transform resolution |
| Chunk indexing | re_chunk_index | reuse if/when indexed project data needs it |
| Chunk layout optimization | re_chunk_optimizer | reuse rather than inventing storage compaction |
| RRD persistence encoding | re_log_encoding / re_log_msg | reuse for Rerun-native persisted data where appropriate |
| Entity paths, timelines, store ids | re_log_types | reuse |
| URI parsing for Rerun resources | re_uri | reuse in its domain |
| Arbitrary project/plugin data | custom components / AnyValues / Arrow | reuse; adding custom data does not require rebuilding Rerun |
| Data reshaping/remapping | re_lenses / re_lenses_core | reuse when the problem is data transformation, not behavior evaluation |
| MP4-to-Rerun ingestion | re_mp4_reader | optional ingestion helper only; FFmpeg remains media codec/container owner |

This makes Rerun a strong candidate for Motolii's **data backplane**: entity/component identity, evaluated values, timelines, history, queries, transforms, and Rerun-native persistence/transport.

### Important boundary: observation is not evaluation

Rerun's core temporal query semantics are latest-at and range queries over values that already exist in the store.

Do not reinterpret those queries as Motolii's animation/behavior evaluator.

For example:

output@10s -> Echo asks for 8..10s -> upstream time-remap asks for 7.5..9.5s

still requires a Motolii Host temporal resolver because some requested values may not have been evaluated yet. Rerun can store/query the resulting evaluated values and may serve cached observations, but it does not define how Bezier, Spring, Echo, simulation, or arbitrary third-party behavior produces them.

Therefore the likely irreducible Motolii-owned temporal responsibility is deliberately narrow:

**output time -> recursively resolve declared input-time requirements -> evaluate missing values -> publish/cache evaluated state**

Do not build a second entity database, history database, transform hierarchy, dataframe/query layer, or generic component system around this resolver.

### Custom vocabulary after host freeze

Rerun explicitly supports user-defined/custom data backed by Arrow without rebuilding Rerun. This is important to the one-hour acceptance condition.

New Motolii/plugin vocabulary should first attempt to exist as data/components supplied from user/plugin code rather than requiring a native Rerun fork or a new Motolii world schema.

This does not imply that the Rerun Viewer must understand or render every custom component. Motolii's own operations may consume those components directly.

### Video boundary

Rerun already provides AssetVideo, VideoFrameReference, VideoStream and re_mp4_reader. Reuse its useful **time/reference representation** where appropriate.

Do not move codec/container ownership from FFmpeg to Rerun:

- AssetVideo is currently MP4-oriented.
- VideoStream is marked unstable.
- Rerun itself uses FFmpeg in some MP4 ingestion/transcode paths.

The clean split remains:

FFmpeg = media decode/encode/container authority.
Rerun = time-indexed entity/component/reference data.
Motolii Host = temporal request resolution connecting them.

### Do not automatically reuse Viewer ownership

Viewer capabilities are not automatically Host capabilities. Rerun Viewer layout, Blueprint UI, egui application state, and Rerun's own renderer are not adopted merely because they exist.

The audit target is reusable data/store/query/transform infrastructure. Flutter remains Motolii UI, and Motolii visual execution remains the native-contributor -> shared GPU-frame direction.

### Revised minimal host hypothesis

After this audit, the Host should be treated suspiciously if it grows much beyond connections among:

- temporal requirement resolution;
- Rerun data backplane;
- external asset/media/material/vector owners;
- shared GPU frame/resources;
- dynamic load/reload boundary.

Anything else must pass the infrastructure replacement gate above.


## Architecture Freeze v0: resolve the remaining implementation choices before coding

The following choices are intentionally decided now so implementation agents do not invent local infrastructure merely because a decision was left open. Revisit one only when a concrete fixture proves the frozen choice cannot satisfy the product.

### Ownership firewall

No dependency may acquire ownership of Motolii Host Time, the Stage/composition, or the Motolii world merely because it offers convenient NLE, scene, renderer, or game-engine features.

Specialists may own their completed local responsibility and return data/resources/results across a narrow boundary.

Do not adopt MLT, GStreamer Editing Services, a game engine, a second scene graph, or a second world-owning renderer as the composition/timeline authority.

Ordinary NLE editing gestures such as trim, split, ripple, snap, markers, duplicate, delete, copy/paste, mute/solo and in/out are authoring-state transformations/UI interactions, not justification for another timeline/composition engine.

### 1. Undo/Redo: command journal over authoring edits

Freeze the user-visible edit model as reversible authoring commands/transactions.

Each UI edit produces one logical transaction with enough before/after data to invert it. Undo applies the inverse transaction; redo reapplies the transaction. Group continuous gestures such as dragging or scrubbing a value into one transaction.

Rerun stores/evaluates resulting world state; it is not the Undo manager. Do not derive Undo by rewinding playback time or by treating Rerun Clear as authoring Undo.

Do not introduce SQLite solely for Undo in the first host. SQLite Session remains a future replacement candidate only if authoring state is later intentionally stored in SQLite.

### 2. Autosave and crash recovery: append journal + atomic checkpoint

Freeze recovery semantics as:

authoring transaction -> append recovery journal -> periodically write an atomic project checkpoint -> truncate/rotate journal only after the checkpoint is durable.

On startup, load the last valid checkpoint and replay later journal entries.

Use operating-system atomic replace/rename semantics for checkpoint publication. Do not build a background autosave engine or database solely for this.

Undo transactions and recovery journal entries should share the same edit transaction representation rather than forming two unrelated systems.

### 3. Project format: small manifest/document + external/native resources; RRD is data, not the whole project container

Freeze the project as a directory/package with a small versioned human-inspectable manifest/document describing authoring structure and references.

Large media remains external or managed through OpenAssetIO references unless explicitly embedded. MaterialX, WGSL, glTF, SVG and similar native resources remain native files. Rerun/RRD may persist evaluated/time-aware data and caches where useful, but .rrd is not the sole opaque Motolii project format.

The project format must not duplicate codec, material, vector, 3D, or asset-manager formats already owned by specialist systems.

### 4. Evaluation cache: disposable content-addressed derived cache

Freeze evaluation cache identity as a deterministic key derived from:

operation identity/version + normalized parameters + requested output time/sample request + identities/hashes/versions of required inputs + relevant quality/render settings.

The cached value is derived and disposable. It is never authoritative project state and may be deleted completely without changing the work.

Rerun query caches remain Rerun's concern. Motolii's evaluation cache only covers expensive evaluated results that do not yet exist merely by querying Rerun.

Start with memory cache and optional disk-backed blobs; do not create a cache database/schema unless measured requirements force one.

### 5. Stateful simulation: deterministic checkpoints owned by Host time resolution

Freeze the correctness model for stateful operations:

- an operation declares fixed-step/sample requirements and serializable/restorable state capability;
- the Host chooses/owns checkpoint times and checkpoint cache;
- arbitrary seek restores the nearest valid checkpoint at or before the required time and deterministically advances to the requested time;
- checkpoints are derived/disposable cache, not authoring truth;
- an operation must not require that playback happened immediately before the requested frame.

Feedback that semantically reads previous rendered results uses the same host-owned temporal requirement/checkpoint rule rather than a hidden playback-only history buffer.

### 6. Dynamic operation/effect contract: data manifest + WGSL first, external process/WASM only when required

Freeze the first extension boundary as files discovered at runtime, not native host code:

- manifest: stable id, display metadata, parameters, input/output resource kinds, temporal requirements/capabilities, required GPU features and shader entry points;
- WGSL/resources: loaded and hot-reloaded at runtime;
- Inspector controls are generated from manifest parameter metadata;
- adding/editing an effect must not require Flutter/Rust/native host rebuild.

Do not design a universal plugin SDK before a fixture requires executable CPU-side extension logic.

If CPU-side third-party execution becomes necessary, prefer a sandboxed WASM boundary before native dynamic libraries. Native plugins are a last resort for capabilities that cannot be expressed through existing specialists, data, WGSL, or WASM.

### 7. MaterialX runtime boundary: generate/cache WGSL outside playback, execute only wgpu resources during playback

Freeze MaterialX/OpenPBR responsibility as material description plus shader generation.

MaterialX documents remain native source assets. Shader generation/translation occurs on material change/load/build time, never in the hot playback path. Generated WGSL and pipeline/resource-layout metadata are cached as derived artifacts.

During playback the Stage sees only the generated shader/pipeline bindings and frame resources required to evaluate the material. It does not run MaterialX graph interpretation per frame.

A MotoliiMaterial class hierarchy or proprietary material graph is forbidden unless a concrete fixture proves MaterialX/OpenPBR plus bounded bindings cannot represent the required work.

### 8. Stage transparency and advanced visual interaction: explicit frame-resource contracts, starting with weighted blended OIT

Freeze the Stage rule: advanced interactions are solved by explicit GPU resources/passes, never by installing another renderer/world.

Baseline transparency for overlapping ordinary transparent contributors uses weighted blended order-independent transparency when simple opaque/depth composition is insufficient.

Refractive/behind-color materials may request an explicit resolved behind-color/scene-color resource and depth/normal resources. Effects/materials must declare these resource needs rather than reaching into hidden renderer state.

If a fixture such as intersecting translucent lyric walls proves weighted blended OIT insufficient, the next escalation is a bounded per-pixel fragment-list/depth-peeling style resource contract for that fixture, not a replacement renderer.

Shared lighting/shadow/environment/emissive inputs likewise become explicit frame resources. Their existence does not create a second scene/world owner.

### 9. Authoring document: versioned declarative edit document; evaluated state is published to Rerun

Freeze the separation:

Authoring Document = what the user meant/edited.
Rerun = evaluated time-aware world/state/history/query.
Stage = visual execution of evaluated/native contributions.

The Authoring Document stores stable ids, hierarchy/grouping, source/resource references, placement/time mapping, operation/effect declarations, parameters/animation authoring data, and presentation/editor metadata required to reopen the work.

It must not duplicate evaluated per-frame component history already suitable for Rerun, decoded media, generated shader binaries, proxy media, thumbnails, or other derived caches.

Authoring edits are pure document transactions. The Temporal Resolver evaluates the document at requested time(s) and publishes resulting state/components/references to the Rerun backplane.

### Frozen decision hierarchy for agents

When implementing a feature, choose in this order:

1. connect a frozen specialist owner;
2. express it as authoring data/edit transaction;
3. express it as Rerun data/query/transform;
4. express it as runtime WGSL/frame-resource contribution;
5. add a bounded adapter between those owners;
6. only then report a missing Motolii-owned primitive.

Do not silently proceed to step 6. A new Motolii-owned primitive requires a failing concrete fixture and an architecture review.


## The one-hour rule is an architecture acceptance test

The one-hour target is not a delivery deadline and must never justify cutting technologies, fixtures, or product requirements. It is a forcing function for architectural clarity.

A mature-technology-aware developer or set of parallel agents should be able to connect the complete host from an empty repository in roughly one hour because Motolii owns very little code and very little behavior. If the build cannot fit that shape, first suspect duplicated ownership, an unnecessary abstraction, or a missed mature owner rather than reducing scope.

The desired result is not "a small demo built quickly." It is a complete thin host whose source tree and ownership can be understood almost immediately.

### One hour is also a test-iteration constraint

The one-hour target assumes that Motolii does not repeatedly re-prove behavior already owned and tested by mature dependencies.

When Motolii connects FFmpeg decoding to a GPU resource, Motolii does not write codec correctness tests. When it connects ThorVG output to the Stage, it does not re-test SVG parsing. When it executes generated MaterialX/WGSL, it does not reproduce the upstream material system's test suite.

Motolii tests its boundaries and its own irreducible semantics, not the internals of its dependencies.

Therefore a large new Motolii unit-test suite is an architecture smell as well as a time cost. It often indicates that Motolii has silently acquired semantics that should belong to an existing specialist.

This is especially important for code agents: an agent must not justify a new subsystem merely by writing a local implementation and then a test suite whose cases it chose itself. Passing self-authored tests proves only that the implementation matches those selected cases; it does not prove that Motolii should own the responsibility.

### Prefer fixture-based electrical continuity tests

The primary tests for the one-hour host are a very small number of real end-to-end fixtures:

- **Smoke:** one work contains decoded video, ThorVG text/SVG, 3D/spatial data, an OpenPBR/MaterialX material, a runtime WGSL effect, temporal Echo, and final frame output.
- **Arbitrary seek:** render times in a non-sequential order such as 0, 10, 3, 8 and obtain the same results as sequential evaluation.
- **Save/reload:** edit, save/checkpoint, terminate, reopen, and reproduce the same work/frame.
- **Runtime extension:** while the native host is unchanged, add or modify a WGSL effect plus its manifest and observe the result without rebuilding the host.
- **Export:** the same evaluated work reaches FFmpeg output without introducing a second composition/time authority.

These are closer to electrical continuity checks than subsystem reimplementations: they prove that completed boxes actually communicate across Motolii's thin lines.

A host rebuild required by the runtime-extension fixture is a failure.

### Unit tests are reserved for irreducible Motolii behavior

Small focused unit/property tests are appropriate only for behavior Motolii genuinely owns, especially the Host-Time temporal resolver and other proven bounded adapters with non-trivial semantics.

Do not create broad test suites for codecs, vector parsing, material semantics, asset management, color transforms, image formats, editorial interchange, or other behavior already owned upstream.

A useful warning heuristic is:

large Motolii unit-test surface
-> large Motolii-owned semantic surface
-> likely duplicated ownership or reinvention.

This is a warning, not a numerical ban: a concrete Motolii-owned invariant may deserve extensive testing if a fixture proves that invariant is truly ours.

### Failure protocol during the hour

If an implementation is taking too long, do not remove MaterialX, ThorVG, Rerun, FFmpeg, temporal effects, persistence, seeking, export, or other required capabilities merely to meet the clock.

Instead stop and classify the obstruction:

1. the direct connection/API is not yet understood;
2. the selected dependency is the wrong owner;
3. Motolii has accidentally taken ownership of behavior already implemented elsewhere;
4. a genuinely missing Motolii primitive has been exposed.

For cases 1-3, simplify or replace the connection. Case 4 requires a concrete failing fixture and architecture review before implementation.

### One-hour readability criterion

At the end of the hour, success includes architectural legibility:

- the source tree can be explained in about a minute;
- every Motolii source file has an obvious reason to exist;
- ownership fits on one diagram;
- the same semantic type is not redefined at multiple boundaries;
- Manager/System/Registry abstractions have not proliferated;
- custom schemas/components are exceptional rather than the default;
- adapters primarily translate or pass resources between mature owners;
- adding ordinary visual work does not require rebuilding the native host.

The target feeling when opening the Motolii core is: "is that really all of it?"

### Agent instruction

Do not optimize for the number of tests written or local test pass percentage. Optimize for deleting Motolii-owned behavior by connecting mature owners directly.

If you are about to spend a significant fraction of the one-hour budget implementing and iterating on tests for a new subsystem, stop before implementing it and search for the mature system that already owns that responsibility.

Do not reduce the required fixture to make the one-hour target pass. Reduce Motolii.
