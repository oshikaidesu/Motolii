# Standard Glass is shared-backdrop transmission (2026-09-23)

Ruling by the user (2026-09-23): the meaning of standard Glass / Transmission is being redefined, not to make it faster, but to give it the right role.

**New meaning:** standard Glass refracts the non-glass picture beneath it, and never refracts another glass.

There is one transmission input per view (and per picture made during preparation). All standard glass reads that one input. When the stack is used for effects that read the view's own image — sequential or recursive refraction, feedback — it goes through WGSL / a custom effect that explicitly reads the composite below. This is a split of roles, not a quality mode.

## 1. The higher-level policy (2026-09-09, "game-engine lies")

- [Glass raster research](2026-09-09-glass-raster-research.md) and [Adopting entertainment rendering](2026-09-09-entertainment-rendering-adoption.md):
  - Physical accuracy is not required; a lie is fine.
  - Look in game engines for a lie that still holds when a Repeater multiplies the number of copies.
  - Do not use accurate physics as the only yardstick; use an approximation that keeps the intended impression and motion.
- Repeaters use shared geometry, programs and images, which cuts unnecessary background copies.

## 2. How ordered transmission got fixed in place

- [Shared reflection execution](2026-09-09-shared-reflection-execution.md) item 3, "keep the existing order for transparent surfaces that read the backdrop", was a conservation condition. Its purpose was to avoid changing transmission's meaning at the same time as reflection became shared.
- It was then promoted into regression tests (`overlapping_glass_reuses_one_backdrop_without_removing_transmission_steps`, "3 transmission levels") and into the acceptance criteria for the next step.
- The result is that ordered transmission became fixed as if it were standard Glass's permanent meaning. That lower-level condition does not take precedence over the higher-level policy in section 1.
- The run-cutting introduced on 2026-09-08 ([Glass refracts what is behind it](2026-09-08-transmission-backdrop.md)) is no longer cut per glass layer, as described in section 3. The backdrop mip pyramid and the fork's transmission shading (binding 7/8) are kept as they are.

## 3. Why it changes, and what it follows

Current source was checked for the renderers that handle many transparent objects in real time:
- **three.js WebGLRenderer.** `renderTransmissionPass` renders the opaque objects once per camera into `transmissionRenderTarget`, with mips.
- **three.js WebGPU.** `ViewportTextureNode` makes one framebuffer copy per render. All transmissive objects sample it at a mip level set by roughness (`applyIorToRoughness`), with 3 samples for dispersion.
- **Godot and Filament.** One screen copy per view after opaque (not re-verified in source).
- **In all of these, glass does not see other glass.** three.js excepts only each object's own back face.
- **Bevy.** The only exception: with steps = N it makes N copies, and the user explicitly trades copies for depth.

Motolii follows the same idea in its layer stack:
- **The "opaque" picture.** The composite of non-glass layers stands in for the opaque scene. Until the first glass it is the stack itself. After that, only non-glass runs are also laid onto a non-glass picture (`compositor/view.rs` `record_stack`).
- **One transmission input.** It is copied once, with the mip levels the roughest glass in the view needs, and every glass reuses it. It is copied again only when a non-glass layer is added above glass.
- **Glass does not see glass.** Separate glass layers and a Repeater's copies follow the same rule. Repeaters are not a special case.
- **Cost.** Transmission inputs never scale with the number of glass layers. They are bounded by how many times glass and non-glass alternate in the stack; for a row of glass (a Repeater) it is one. Glass copies are drawn as one instanced batch in one run.
- **Ownership.** The transmission input reads the view's own image, so it is View-owned (in line with the render orchestration ruling). It is not moved into the prepared world.

## 4. What is lost

- Recursive / multiple refraction, where glass behind other glass shows up refracted twice.
- Sequential transmission, where the second glass reads the result of drawing the first ("3 transmission levels").

## 5. Expressive power that stays with WGSL

- An effect with a backdrop input (`reads_backdrop`) or one that reads the composite (`reads_composite`) reads **the whole composite beneath it, glass included**, and runs on the view's picture (`screen_passes`). Previous composite, sequential processing and feedback (history per view) are all explicit inputs there.
- Expensive processing is something the author asks for explicitly. Standard Glass stays real-time oriented.

## Tests

`engine/frame_graph/tick_tests.rs`:
- `glass_layers_share_a_backdrop_however_many`: 1, 2 or 5 separate glass layers give 1 input; a picture between glass layers gives 2.
- `repeated_glass_is_one_transmission_input_and_one_batch_and_previews_as_it_exports`: Repeater ×1/10/100 gives 1 input and at most 3 runs, and preview equals export.
- `overlapping_glass_refracts_the_picture_below_not_the_other_glass`: where glass overlaps, the result equals the upper glass on its own.

`engine/reflection_tests.rs`:
- `overlapping_glass_copies_read_one_shared_backdrop`: this replaces the old test.

Explicit sequential processing is covered by `render/feedback_is_a_recurrence_from_the_in_point.rs::a_chain_that_reads_below_is_also_a_recurrence` and the background-reading effect tests.
