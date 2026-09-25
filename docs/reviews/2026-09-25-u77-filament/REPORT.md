U77 comes out at the middle verdict: **sufficient for posters with a few added inputs or shader features**. With no lights or environment placed, a Filament-based material plus a built-in studio makes chrome, glass, plastic and extruded type read as product shots on white, mid grey, blue and black. The branch `claude/u77-filament-spike` is committed locally at `e28cc7503` in `explore/U77/wt` (not pushed or merged); `MOTOLII_MATERIAL=filament` switches it on.

The tool refused to let me write `U77/REPORT.md`, so its content is below and needs saving. The only files I wrote in `explore/U77/` are `NOTES.md` (which circuits I claimed), `progress/`, `scripts/`, `port/` and `render.sh`.

## What changes the picture
- **Current Motolii, nothing placed:** it falls back to simple fixed lighting, so the chrome and glass spheres render as the same matte white ball.
- **Built-in studio environment:** used whenever no environment is bound. Six soft boxes (key, overhead, two rim strips, fill, front) over a light sky and grey floor, fixed to the camera. Changing the background colour doesn't touch the lighting. U76 found the same thing; like U76's material, mine includes its own copy.
- **Glass edges:** where the ray through glass bends sharply, it shows the studio instead of the flat backdrop. Glass gets a thin dark edge on white and a bright rim on black. This is also U76's finding.
- **Filament-specific gains**, small but visible on chrome and plastic:
  - multiscatter energy compensation with Filament's real lookup table;
  - clear coat, which gives plastic a crisp soft-box glint;
  - Filament's highlight roll-off ("PBR Neutral"), applied only to the object's own light, not to what shows through glass;
  - Filament's solid-sphere refraction with its four-wavelength dispersion.

## What was ported, and how
| Piece | Route |
|---|---|
| GGX distribution, correlated visibility, Kelemen, Schlick Fresnel, Lambert, spherical-harmonics irradiance, dominant reflection direction, sphere refraction, Lagarde specular AO, multi-bounce AO, dispersion matrices, PBR Neutral tone mapper | **naga's GLSL frontend** (naga-cli 29.0.4). About 104 Filament lines copied verbatim plus 8 glue lines, wrapped in a stub `main`; it translated on the first try. Output is 282 generated WGSL lines, names prefixed `fil_`. |
| Pixel parameters, the environment-lighting order, clear coat, refraction, dispersion loop | Hand-ported, about 50 lines. They depend on Filament's material and frame structs, not on anything naga couldn't translate. |
| Multiscatter DFG table (32×32, 1024 samples) | Re-implemented from Filament's `CubemapIBL.cpp` in Python and checked against brute-force integration. Baked as a WGSL constant, so no new texture binding. |
| Spherical-harmonics irradiance of the studio | Baked the same way (Filament's form, Lambert and 1/π included). |

Nothing failed to translate. The specular AO compiles but does nothing, because Motolii has no AO source.

**Code size:**
- Motolii glue in `material_filament.wgsl`: about 88 of its 138 lines. The studio is about 35, the table lookup 12, lighting routing 12, glass-edge sampling 9.
- `surface_program.rs`: +18 / −2 for the switch.
- Generated: the naga module (282 lines) and the baked data (46 lines).
- Offline tooling: `bake.py` (125 lines) and `extract.py`.
- No new GPU resources or passes.

**Where the inputs come from:**
- Material values come from the Glass effect.
- Thickness is the instance scale.
- Refraction reads Motolii's existing backdrop texture.
- With an environment bound, lighting uses re_renderer's existing environment and irradiance lookups; without one, the studio and baked data.

## Pictures (`explore/U77/progress/`)
- `06_final_u77_fixture_4rows.png` — **the main sheet**: chrome, glass, satin puck and extruded "POSTER" on white, grey, blue and black. Rows are Motolii now, Filament, Filament + clear coat, and + Contact Shadow.
- `05_u76_fixture_motolii_vs_u76_vs_u77.png` — U76's own fixture with three columns: Motolii now, U76's Bevy stack, and mine.
  - The two stacks are close because they share the studio and glass-edge ideas.
  - Mine has higher-contrast chrome and glossier plastic; U76's plastic face is lighter.
- `04_glass_garden_motolii_left_filament_right_vs_U72_three.png` — with Filament, the glass disc and orbit spheres get a clean bright rim. The remaining gap to three.js is the whole-frame image (bloom and tone mapping, a reflective floor, cast shadows), not the material.
- `07_hdri_bound_motolii_vs_filament.png` — a bound environment image replaces the background colour, as U76 also found. That's a design question I left for the user.

## Frame time (M4, 1080p, 60 moving frames, release build, includes frame readback)
| Scene | Motolii | Filament + clear coat |
|---|---|---|
| Full fixture | 62–66 ms | 65–69 ms |
| Full fixture + Contact Shadow | 71–72 ms | 78–80 ms |
| Without the extruded-text layer | 13.4–13.7 ms | 15.1–18.0 ms |

- The material costs about 1.5–3 ms, and the scene without text stays near the 16.7 ms target.
- The full fixture misses 60 fps because of the extruded text: it is re-prepared every frame whenever anything in the comp moves (46.5 ms), with either material and even when the text itself is still.

## Features still needed
- **Clear-coat knob on Glass** (about 10 lines). The shader is already written; the spike drives it with `MOTOLII_FILAMENT_CLEARCOAT` because Glass has no input for it. With it on, the extruded type gets a coat too, which a per-object knob would avoid.
- **Contact Shadow:** U76's 79-line effect, copied into my branch unchanged. Glass still casts a dense shadow with it.
- **Emissive input** (about 5 lines).
- **Automatic bloom** after compositing: no new shader.

**Controls to expose to users:** colour, roughness, metallic, transmission and refraction per object, with dispersion as an advanced control; new clear-coat and emissive knobs; contact-shadow strength per layer. No lighting controls: the studio applies whenever no environment is bound.

## Bugs found, not fixed
- Extruded text lands with its top-left at Position minus the comp centre.
- Letter holes (P, O, R) show hatching, with both materials.
- Extruded text is re-prepared every frame, at 46 ms.
- A solid hex fill doesn't reach 3D-projected shapes; a `linear-gradient` of one colour works.

## Reproduce
- `explore/U77/render.sh <script> <outdir> motolii|filament [frames]` uses a copied watch-profile binary that reads the shader files from disk, so shader edits need no rebuild.
- The fixture scripts come from `explore/U77/scripts/gen.py`.
- `bake.py` and `extract.py` regenerate the two generated modules; `naga-cli` is installed under `explore/U77/tools/`.

Files are in `explore/U77/wt/motolii/crates/motolii-render/`:
- `vism/material_filament.wgsl`
- `vism/material_filament_naga.wgsl`
- `vism/material_filament_baked.wgsl`
- `vism/contact_shadow.wgsl`
- `src/compositor/effects/surface_program.rs`