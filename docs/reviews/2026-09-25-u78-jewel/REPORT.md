I've fixed all five Part I bugs (plus two more I found) and built the jewellery circuits, but Glass Garden does not reach 60 fps. The premise that extruded text was its main cost turned out to be wrong: Glass Garden contains no extruded text. The re-preparation fix makes still extruded text essentially free (the U77 fixture dropped from 63 to 2.5 ms), while Glass Garden only went from 28 to 23 ms. Stopping point: the transparent objects now sparkle like a jewellery ad with nothing placed. Nothing is pushed or merged into the base branch.

**Branches**
- `claude/u78-fixes` at `explore/U78/fixes`: 5 fix commits, each with a regression test, mergeable on its own. The base branch has since gained one docs commit (the U78 charter); it doesn't conflict.
- `claude/u78-jewel` at `explore/U78/jewel`: branched from U77 (`e28cc7503`), with the fixes merged in, then 6 spike commits.

The tool setup here discourages writing report files, so the full report is below rather than in `explore/U78/REPORT.md`.

## Part I: bug fixes (`claude/u78-fixes`)

1. **Extruded text re-prepared every frame.**
   - **Cause:** the node that shapes text always re-ran every frame and always read the global layout. So every text got a new value each frame, the renderer saw "its content changed", re-rastered it, missed the extrusion cache and rebuilt the solid. It was GPU time plus freeing large textures, not CPU work.
   - **Fix:** the node now only reads the layout for a child layer (the only case where a layout can reach it). It's treated as time-independent unless its text track, a style property or the layout actually varies. A static text also skips the glyph-transition node. No new cache: this is the existing time-dependency mechanism.
   - **Test:** `a_still_extruded_text_is_not_prepared_again_when_another_layer_moves`. Another layer moves and the text itself tilts; nothing is re-prepared, the same solid is reused, and the picture matches a fresh engine.
   - The harness now prints p50/p95/worst, and `MOTOLII_TIMING_WINDOW=1` times the window path (no readback, GPU waited each frame).
   - **Files:** `frame_graph/text_program.rs`, `compositor/extrude.rs`, `ui/native/src/editor/script.rs`.
2. **`.fill("#hex")` renders white.**
   - **Cause:** a body bug, not the harness. A picked colour is stored as the `shape.fill_color` property (the brush is only its default), but the render path fed shapes to the renderer without applying any `shape.*`/`fill.*` property. So size, stroke-width and gradient-stop properties were also dropped, on every projection.
   - **Fix:** the shape node now takes those properties as inputs and applies them the same way the old resolver did.
   - **Test:** `a_shape_draws_its_fill_color_property_over_the_documents_brush` (geometry matches the old resolver; a keyed red→blue colour renders on 2D, 2.5D and 3D). **File:** `frame_graph/content.rs`.
3. **Text offset by the comp centre.**
   - **Cause:** text is laid out on the comp canvas and its anchor is measured there. The picture route (extrude, and also Blur and other neighbourhood effects) rastered the glyphs on their own tight bounds instead.
   - A blurred title could leave the frame entirely.
   - **Fix:** both routes raster on the comp canvas.
   - **Test:** `text_as_a_picture_or_a_solid_lands_where_its_outline_does` (it failed with a 99 px offset before). **Files:** `render_lowering.rs`, `render_graph/work.rs`, `engine/texture/sources.rs`, `engine/frame_graph_scene.rs`.
4. **Hatching in letter holes (P, O, R).**
   - **Cause:** each wall was oriented by its own contour's winding, so hole walls faced into the solid and a bevel shrank the hole instead of rounding its rim.
   - **Fix:** a contour inside an odd number of others is treated as a hole and faces the other way.
   - **Test:** `a_hole_faces_into_itself_and_its_rim_rounds_away_from_it` (both winding conventions). **File:** `compositor/extrude.rs`.
5. **Found along the way: glass that's first in a view refracted nothing.**
   - **Cause:** the background only existed inside the first draw, so a scene made only of glass showed the studio instead of the poster behind it. This is what made the whole transparent set grey.
   - **Fix:** in that case the glass reads a canvas filled with the background.
   - **Test:** `glass_alone_over_the_background_reads_the_background`. With the stock material this is latent; it shows under the built-in studio. **File:** `compositor/view.rs`.
6. **Contact Shadow too dense on glass** (fixed on the jewel branch, since the effect only exists there).
   - **Fix:** a new "Light Through" input, left on auto, is filled by the host from the layer's own surface: the transmission the Glass effect already declares, × (1 − metallic). The shadow thins by 80% of it.
   - **Files:** `engine/translate.rs`, `vism/contact_shadow.wgsl`.
7. **Small material inputs** (jewel branch):
   - **Clearcoat:** a new Glass input that drives Filament's clear-coat layer on any body, glass included.
   - **Emission:** a new Glass input (surface colour × amount), in both materials.
   - **Bloom:** instead of a new bloom renderer, a Glint shelf effect that rides the existing effect path; see Part II.

Test runs: the render suite has 8 failures and the UI suite 5; both sets fail identically on the base branch. Nothing new fails on `claude/u78-fixes`.

**Glass Garden timing** (M4, 1080p, p50 / p95 / worst):

| Path | Before | After |
|---|---|---|
| Window path, 400 frames | 28.01 / 28.86 / 63.92 ms | 23.43 / 23.84 / 47.88 ms |
| Readback path, 300 frames | 54.95 / 60.11 / 73.19 ms | 48.18 / 53.38 / 60.46 ms |
| U77 fixture (still extruded text), window path | 63.09 / 64.41 / 68.83 ms | 2.54 / 3.17 / 16.92 ms |

The harness's readback path also renders at export quality, so it isn't a 60 fps measure.

**Why Glass Garden is still above 16.7 ms:** it is GPU-bound. Each full-screen 2D shape layer costs about 4.5 ms because the mesh shader shades every MSAA sample. The two Prism Orbit layers alone are about 11 ms, the glass flowers about 6.5 ms. Per-sample shading is a recorded decision (2026-09-09, kept for edge quality after you compared the images), so I didn't touch it. The cheapest candidate would be centroid shading for flat, unlit 2D layers, but that is your call.

## Part II: jewellery circuits (`claude/u78-jewel`, `MOTOLII_MATERIAL=filament`)

All circuits live inside the existing surface shader, plus one shelf effect: no new renderer, no copy of the scene graph. The research audit covered Spectral-Glass (MIT), three.js and drei (MIT), Filament and Babylon (Apache-2.0), Godot (MIT) and FastStarGlow (BSD-3). diamond-webgl is GPL-3.0 (and its source is gone) and Blender is GPL, so neither was ported. No code was copied into U78; everything is re-derived or reuses what U77 already brought in.

| Circuit | Adopted? | Source / licence |
|---|---|---|
| Fresnel inside the stone (exact form; total internal reflection keeps all the light) | Yes | Textbook physics, re-derived |
| Internal path through up to 3 far-side interfaces, splitting light at each exit | Yes (the biggest gain) | Energy-split idea from diamond-webgl, re-derived, no code |
| Far side, slabs: the back cap, bouncing between caps along the extrusion's axis | Yes | Filament's thin-slab concept (Apache), own code |
| Far side, solids: the bounding ball around the object's centre, cut into virtual facets (16 around × 3 tiers) on flat-faceted surfaces | Yes | Motolii-specific; idea after the "normal cubemap" gem technique, no code |
| Light tent: the studio darkened, plus two rings of small hard strip lights, seen only by light leaving the stone after travelling inside | Yes (the sparkle) | Motolii-specific; follows public advice from webgi and Light Tracer |
| Dispersion: Filament's four wavelengths on faceted stones, a cheaper single-path red/blue split on slabs and smooth glass | Yes | Filament K matrices (Apache, already in U77); split idea from Spectral-Glass (MIT) |
| Beer-Lambert absorption, per the object's own size, tinted by its colour | Yes (default 1.0) | three.js / Filament formula |
| Rough transmission | Kept as in U77 | — |
| Glint: star glare on whatever is brighter than its local neighbourhood, two streak axes at half resolution | Yes (the "ad" look) | Kawase GDC 2003 / FastStarGlow (BSD-3) idea, own code |
| Three full red/green/blue paths | Removed | Looked like the four-wavelength version but less vivid, saved only 0.6 ms |
| Caustics | Not needed | Nothing cheap exists per pixel |

**Code size:**
- Motolii-specific lines on the spike: +372 / −22.
  - `material_filament.wgsl` +188.
  - `glint.wgsl` 99.
  - About 60 lines of Rust, most of it for handing the object frame to the shader.
  - Small edits to `glass.wgsl`, `material.wgsl` and `contact_shadow.wgsl`.
- Upstream-derived lines added in U78: none (U77's 282 generated lines are reused).

**GPU resources and passes:**
- The jewel path adds none.
- Glint adds, per layer, four float targets (1/16 size and 3 × 1/2 size) and five full-screen passes.
- The host writes the solid's kind and frame (centre, radius, rotation, half depth) into spare surface slots per instance.

**Frame time per stage** (window path, p50 ms; `progress/09`):

| Stage | Transparent set | + Glint ×6 | Poster grid | Glass Garden |
|---|---|---|---|---|
| Current Motolii | 2.5 | 8.8 | 2.8 | 24.7 |
| U77 Filament + studio | 5.0 | 11.9 | 4.5 | 28.8 |
| + jewel, four wavelengths everywhere | 12.1 | 19.7 | 7.8 | 44.9 |
| + jewel, single-path split everywhere | 9.0 | 16.5 | 6.6 | 34.9 |
| **U78 default** | 12.5 | 20.2 | 6.4 | 35.2 |

The first version of the light tent cost +13 ms; making it look up only the nearest light per ring cut that to about 5.5 ms. Glint costs about 1.2 ms per layer. Putting one Glint on a group of all stones is much worse (64 ms, because the group becomes a plate recomposed every frame), so Glint stays per layer. For 60 fps, use it on the hero stones: about 3 glinted stones fit.

**Compared with current Motolii and U77:**
- The diamond and crystal now read as cut stones, with kaleidoscopic facets, blue/orange fire and star glints.
- Clear type and logos read as thick acrylic with bright internal-reflection edges.
- It works on white, grey, blue and black with nothing placed (`progress/05` and `06`).

**Compared with U72 three.js:** Glass Garden changes little, because it's a scene of dark petals; the remaining gap is still image formation (reflective floor, cast shadows, HDR bloom) (`progress/07`).

**Controls to expose:**
- Glass: Colour (the fill), Roughness, Metallic, Transmission, Refraction, Dispersion (advanced), Clearcoat, Emission.
- Glint: Intensity, with Threshold, Length and Angle as advanced.
- Contact Shadow: Strength, with Light Through on auto.
- No lighting controls. Slab vs solid and faceted vs smooth are detected automatically.

**Environment vs background: this would have to enter the body permanently, and it needs your decision.**
- In the spike, a bound environment only lights, reflects and refracts; the camera sees the composition background (`progress/08`).
- The switch sits in one place, `environment_visible_to_camera` in `view.rs`, ready for a future "Visible to Camera" setting. `MOTOLII_ENV_VISIBLE=1` restores the old behaviour.
- Three existing tests encode "the environment fills the background" and fail on the spike by design; they pass with the flag. Adopting this changes an existing contract.

**Known limits:**
- A smooth arc seam on the diamond where exiting light switches between the backdrop and the tent.
- The prism reads as a small kaleidoscope rather than a rainbow prism.
- The smooth glass sphere barely changes.
- The render view is 8-bit, so there's no true HDR bloom; Glint's local-contrast bright pass is the workaround.
- Found, not fixed: the glyph-transition node moves the glyph outlines but never rebuilds the shapes the renderer draws from, so transitions may not render.

## Pictures (`explore/U78/progress/`)
- `01_text_position_holes_hexfill_before_after.png` — Part I bugs 2–4, before and after
- `02_baseline_transparent_set_motolii_vs_u77filament.png` — the starting point
- `03_internal_bounces_faceted_proxy_light_tent.png` — first jewel stage
- `04_jewel_stage_tent_dispersion_glint.png` — with dispersion and Glint
- `05_poster_grid_current_u77_u78.png` — **poster grid:** current / U77 / U78 / U78 + Contact Shadow
- `06_transparent_set_current_u77_u78.png` — **transparent set:** current / U77 / U78 / + Glint / + Contact Shadow
- `07_glass_garden_current_vs_u78_vs_u72_three.png` — Glass Garden against U72 three.js
- `08_bound_environment_lights_only_top_visible_bottom_background.png` — environment vs background
- `09_frame_time_per_stage.png` — the timing table

Fixtures come from `explore/U78/scripts/gen.py` (gem meshes from `scripts/mesh/gems.py`). Renders use `explore/U78/render.sh`. `MOTOLII_FILAMENT_CONSTS="JEWEL_…=…"` toggles individual circuits without a rebuild.