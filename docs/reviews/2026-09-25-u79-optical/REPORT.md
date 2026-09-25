# U79 report: Optical Lies

The 300 Jewel Repeater runs at 60 fps at 1080p on the M4, and a single Look pass adds the optical lies. Glass Garden does not reach 60 fps: it went from 35.2 to 27.0 ms p50, and the remaining cost is structural (below). `claude/u78-fixes` is ready to merge. Nothing is pushed or merged. The environment is still lights-only, as in U78.

The tool setup here discourages writing report files, so the report is this message rather than a REPORT.md.

## Where things are

**U79 worktree:** `explore/U79/optical`, branch `claude/u79-optical` (from `claude/u78-jewel`, 7 commits).

**Fork patches:** a local clone of the fork at `explore/U79/rerun`, branch `claude/u79-optical`, 3 commits: +20/−4 lines in 3 files. The spike points `[patch]` in Cargo.toml at this clone.
1. A surface program can ask to be shaded once per pixel (`pixel_rate`).
2. The view's main target becomes `Rgba16Float` (one constant).
3. `CpuModel::is_faceted`, which tells whether every triangle is flat-shaded.

**Scripts:**
- Fixtures: `explore/U79/scripts/gen.py`.
- Rendering: `render.sh`.
- Sheets and tables: `tools/grid.py`, `tools/table.py`.
- Timing and ablation: `tools/time.sh`, `tools/ablate.sh`.
- The harness now prints CPU and GPU time apart, and `MOTOLII_TIMING_SPIKES=1` lists slow CPU frames.

**Switches:**
- `MOTOLII_LOOK=off|studio|poster|jewel` (studio when unset).
- `MOTOLII_LOOK_PARAMS="star=0;..."` overrides single strengths.
- `MOTOLII_FILAMENT_CONSTS` toggles material circuits, as in U78.
- `MOTOLII_STONE_PIXEL_RATE=0` turns off the per-pixel shading for small stones and flat faces.

## Step 1

**a) `claude/u78-fixes` is ready to merge.**
- I re-reviewed all 5 fixes. They are narrow bug fixes, each with a test.
- I checked that marking still text as time-independent is safe: its shaping reads time only through the content track.
- Rebased onto the base branch, it is now a straight fast-forward (tip `15e334a66`).
- Test results: the render suite has 8 failures and the UI suite 5 on both the base and the fixes branch, the same tests by name. The fixes branch adds 5 passing tests (338 vs 333).

**b) Flat 2D layers no longer shade per MSAA sample** (`progress/01`).
- **Cause:** a full-screen 2D layer is drawn as a flat mesh whose curve coverage and texture are read at the pixel centre. Shading it 4 times per pixel computed the same value 4 times.
- **Fix:** unlit, flat pictures now shade once per pixel. MSAA still resolves their edges from coverage.
- **Edges:** output is bit-identical (0 pixels differ on the flat poster, the grey set and Glass Garden frame 120).
- **Speed (GPU):** one full-screen layer 5.77 → 2.37 ms. The two Prism Orbit layers 11.4 → 4.7 ms. Glass Garden at that point: 35.19 → 27.90 ms p50 (GPU 30.8 → 23.7).
- The 2026-09-09 per-sample decision still holds for lit surfaces.

## Step 2

**HDR.** The least invasive route was the view canvas format, and it is not an architecture change:
- One fork constant turns the view into scene-linear half float. Motolii canvases then follow it.
- Canvases in that format are read as linear and premultiplied, as the sRGB ones were.
- Lit surfaces keep U77's PBR Neutral shoulder and pass only the light above 1 into the HDR view. Flat white text stays at 1.0 and never glows.
- The output rolls highlights off without clipping to a tint: saturated colours keep their hue (the neon word stays pink), near-neutral highlights go white.
- Cost: +1.2 ms on Glass Garden, about 0 elsewhere.
- Known caveat: MSAA now resolves in linear HDR, so a very bright sample dominates its pixel and diamond facet edges are slightly harder. Fixing that means a reversible tone map around the resolve, which is a fork change.

**One Look pass, once per view** (`compositor/look.rs` + `look.wgsl`):
- One bright-pass pyramid (half to 1/64 size) feeds everything.
- **Bloom:** from the pyramid.
- **Halation:** red, and only around highlights, not over them.
- **Star:** six blades with coloured tips. It grows only from point lights (a light brighter than its surroundings), so a lit area such as a neon word or a softbox in chrome does not grow blades.
- **Anamorphic streak:** also from point lights only.
- **Colour split:** lateral aberration (0.7 px at the corners in Studio), print-plate misregistration (Poster only, 0.8 px, CMY-like slivers at edges and overlaps), and dispersed glare (a white highlight opens into a rainbow rim).
- Cost: 13 small passes per view, about 0.8 ms, constant whatever the scene holds.

**Chromatic response in the material:**
- Glass silhouettes read their reflection at three angles, one per colour, so a bright band on the rim gets a red and a blue edge. It runs only near the rim.
- A seeded per-facet sparkle sends HDR flashes that the Look's star picks up.

**Looks:**
- Default Studio: subtle.
- Poster / Optical: medium, plus plates and streak.
- Jewel / Crystal: strongest stars.
- None of them makes everything rainbow (`progress/04`, `05`, `05b`, `06`).

**Jewel level of detail by size on screen** (the stone's radius in pixels):

| Radius on screen | What it gets |
|---|---|
| 180 px or more | 3 bounces, colour split where the light leaves |
| 90–180 px (hero) | 3 bounces + 4 wavelengths |
| 28–90 px | 2 bounces, split |
| 5–28 px | 1 bounce, split |
| under 5 px | the light tent seen off the facet, plus sparkle |

Each copy carries a stable seed (its place in the layer list) for facet and sparkle variation. There is no pass, Glint or environment work per copy.

**The per-sample decision needs your call.** I extended the per-pixel shading to two more cases:
- Cut stones smaller than a hero on screen.
- The flat caps of extruded solids, unless a field bends them.

In both, silhouettes and facet edges are still MSAA-resolved. What gets slightly more aliased is the kaleidoscope pattern inside a stone: visible when zoomed on a hero (so heroes keep per-sample), not at grain size. This is what makes 300 jewels fit in a frame (27 → 9 ms). It goes beyond the scope of the 2026-09-09 decision, which covered lit meshes, so whether to adopt it is your call.

## 300 Jewel Repeater (`progress/03`, `08`)

1080p window path, 300 frames each (ms):

| Fixture | Total p50 / p95 / worst | CPU p50 | GPU p50 |
|---|---|---|---|
| 1 | 2.68 / 3.14 / 13.71 | 0.48 | 2.17 |
| 10 | 2.91 / 3.20 / 14.54 | 0.53 | 2.36 |
| 50 | 3.73 / 4.08 / 18.46 | 0.66 | 3.06 |
| 100 | 4.70 / 5.12 / 11.08 | 0.83 | 3.85 |
| 300 | **8.79 / 9.23 / 17.58** | 1.53 | 7.21 |
| 300 + 3 heroes | 10.98 / 11.52 / 16.85 | 1.66 | 9.25 |
| 300 + 3 extra-large heroes (~500 px wide) | 14.68 / 15.19 / 20.18 | 1.71 | 12.92 |
| 300, Jewel look | 9.47 / 9.93 / 15.54 | 1.64 | 7.75 |
| 300 on the U78 path | 59.49 / 62.69 / 65.11 | 1.90 | 57.60 |

- The structure does not grow with the count: 1 view run, 1 instance upload, at most 2 draws (the per-sample and per-pixel programs), 13 Look passes.
- GPU time grows with the pixels covered.
- CPU time grows by about 3.5 µs per copy.

## Glass Garden: not at 60 fps (`progress/07`)

300 frames, window path (ms):

| Build / look | Total p50 / p95 / worst | CPU p50 / p95 | GPU p50 / p95 |
|---|---|---|---|
| U78 | 35.19 / 50.18 / 56.24 | 4.37 / 18.96 | 30.79 / 31.38 |
| U79 Look off | 24.56 / 38.67 / 60.76 | 4.16 / 17.80 | 20.42 / 21.16 |
| U79 Studio | **26.97 / 41.75 / 59.02** | 4.69 / 19.15 | 22.28 / 22.89 |
| U79 Poster | 27.20 / 42.22 / 67.14 | 4.68 / 19.25 | 22.57 / 23.20 |
| U79 Jewel | 27.26 / 42.19 / 58.75 | 4.71 / 19.15 | 22.57 / 23.21 |

The glass disc's flat caps now shade once per pixel, which cut the disc from about 9.5 to 3 ms. What remains:
1. **The two Repeater rings, each with its own Glow**, cost about 6 ms GPU plus CPU command encoding, because each ring becomes a plate rebuilt every frame. Fixing this is compositor or plate architecture, which is a stop condition.
2. **A CPU frame of about 19 ms every 2–3 frames.** It is already in U78, so it predates U79, and it sets p95. I flagged it as a separate task.
3. **Shading the wave-bent petal caps once per pixel would save 3.2 ms** (max pixel difference 35/255 over 323 px). I kept them per sample because they are curved.

## Per-circuit ablation (`progress/09`)

GPU p50 ms, Default Studio unless named:

| Configuration | Set | Glass Garden | 300 + heroes |
|---|---|---|---|
| All on | 8.44 | 22.28 | 9.29 |
| Look off entirely | 7.48 | 20.48 | 7.98 |
| Look pass off (material lies kept) | 7.66 | 21.22 | 8.39 |
| Rim dispersion off | 8.20 | 21.24 | 8.97 |
| Sparkle off | 8.52 | 21.77 | 9.06 |
| Jewel LOD off | 8.52 | 23.03 | 16.06 |
| Per-pixel stones and caps off | 8.67 | 28.54 | 27.25 |

Every adopted circuit works on 3D. The screen-level lies (colour split, plates) also work on 2D; bloom, star and halation need light above 1, which a 2D layer cannot produce yet.

**What was added:** 13 passes and about 29 MB of pooled textures per view for the Look, view canvases twice their old size (half float), and a second program per surface effect (its per-pixel twin).

**Motolii-specific code:** +658/−29 lines. That includes `look.rs` (195), `look.wgsl` (183), +103 in `material_filament.wgsl`, 40 lines of the Cargo patch table and 19 of harness changes.

**Sources and licences** (every idea was re-derived; no code was copied):

| Circuit | Source | Licence |
|---|---|---|
| Bloom pyramid | Jimenez 2014, as Bevy writes it | MIT/Apache-2.0 |
| Star | Kawase GDC 2003 | idea only |
| Streak | KinoStreak | MIT |
| Halation | utility-dctls | MIT |
| Lateral aberration | pmndrs postprocessing, Filament flare | Zlib, Apache-2.0 |
| Plate misregistration | print practice, three.js HalftoneShader | MIT |
| Sparkle | after Zirr–Kaplanyan and Deliot–Belcour | idea only |
| Rim dispersion | own | — |

**Not adopted:**
- Thin-film iridescence at grazing angles: it only turned the dark rims olive, with little to see, so I removed it.
- Faking the small stones' interior without a path: it looked like flat pastel, and the real 1-bounce path at pixel rate costs the same.
- Lens ghosts and halftone: not tried in the time I had.

## Needs your decision

1. Adopting per-pixel shading for small stones and flat caps (it extends the 2026-09-09 decision).
2. Where the Look lives as a product setting (a composition-level "Look" choice, and which one is the default), and whether the fork's HDR constant enters the body.
3. A 2D layer has no way to be emissive, so a flat word cannot halate; that would be a new semantic.

## Other findings

- **Test results on `claude/u79-optical`:** the only failure caused by U79 was a test that counts surface programs; I updated it for the twins. Three environment/glass tests fail only in debug builds against the local fork, and fail the same way on `claude/u78-jewel` pointed at the unmodified local fork, so they come from the local-path build, not from U79 code. My watch-profile renders from the git fork and the local fork were bit-identical.
- **Shader NaN:** Metal's fast math turns `pow` of a slightly negative base into NaN and removes `x != x` checks. Guards now test the bits.
- **Pre-existing bug (task flagged):** `star()` shapes render with stair-step holes (the fork's newest curve-fill change), and a `#RRGGBBAA` fill ignores its alpha.

Everything is in `explore/U79/progress/`:
- 01_msaa_flat_layers_pixel_rate_edges_and_timing.png
- 02_hdr_view_and_look_pass_off_studio_poster_jewel.png
- 03_300_jewel_repeater_lod_pixelrate_small_stones.png
- 04_before_after_contact_sheet_set_u78_vs_studio_poster_jewel.png
- 05_flat_text_svg_sheet.png
- 05b_flat_text_edges_x6.png
- 06_jewel_glass_sheet.png
- 07_glass_garden_before_after.png
- 08_300_jewel_repeater_frames_and_scaling.png
- 09_per_circuit_ablation.png