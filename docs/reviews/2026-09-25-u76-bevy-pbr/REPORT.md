**U76 verdict: SUFFICIENT FOR POSTERS WITH A FEW ADDED SHADER FEATURES, but almost none of the gain comes from Bevy.** Swapping bevy_pbr's WGSL in for Motolii's material, with the same environment map and backdrop as today, makes no visible difference. The poster-grade look comes from three small additions that are not Bevy.

I couldn't write `U76/REPORT.md`: the harness refuses report files from a subagent, so its content is below for you to save. The branch `claude/u76-bevy-pbr-spike` is committed locally as `b385cfef1`, not pushed or merged, and `MOTOLII_MATERIAL=bevy` switches the material on. Pictures are in `explore/U76/progress/01…06`. `U76/NOTES.md` lists which circuits U76 took, for U77.

## What changes the picture
- **Bevy BRDF alone does nothing visible.** In Glass Garden (`progress/05`) the Motolii row and the Bevy row look the same. The only parts of Bevy's forward path that Motolii's inputs can reach are either what Motolii already does or effects too small to see:
  - Already in Motolii: the Karis split-sum approximation (Bevy's lookup-table version needs a new binding), environment mips chosen by roughness, and a screen-space backdrop for transmission.
  - Too small to see at poster scale: multiscatter energy compensation, the Frostbite reflection direction, specular occlusion, and Bevy's spiral transmission blur.
  - Never runs: direct lights, shadow maps, clearcoat and anisotropy, because Motolii has no punctual lights and Glass has no inputs for them.
- **Three non-Bevy additions do the work** (`progress/06_final_posters_motolii_vs_bevy_stack.png`, today on the left, the stack on the right):
  1. **Automatic studio environment** whenever no environment is bound: a soft key box, two rim strips, a grey wall and a dark floor, fixed to the camera. About 40 lines of WGSL. Today, with nothing bound, chrome and glass render as the same matte white ball. With it, chrome and glass read on white, grey, red and black, and plastic and type get shape.
  2. **Glass edges show the environment instead of the flat backdrop.** One line: `backdrop.a *= smoothstep(0.08, 0.5, N·V)`. Glass gets a dark outline on white and a bright one on black.
  3. **Contact Shadow**, a new shelf effect in `vism/contact_shadow.wgsl` (79 lines). It darkens the background just below the layer's own silhouette, blurs it at three sizes with Glow's filters, and draws it under the layer. Nothing to place, no receiver layer, no new renderer pass type.
- **Recommendation:** adopt these circuits, not Bevy. Taking Bevy costs about 300 hand-flattened lines, 16 local edits and a metres-vs-pixels unit fix for no visible gain. U77's Filament port claims the same BRDF circuits and adds a baked DFG table, so U77 should own the BRDF. The three additions work with either BRDF.

## Pictures (`explore/U76/progress/`)
| file | what |
|---|---|
| `01_…` | first pass: 4 fixtures × 4 backgrounds, nothing bound; Motolii vs Bevy + studio |
| `02_…` | same scene with an environment image bound: the two materials look the same, and the environment **replaces the background colour** |
| `03_…`, `04_…` | Contact Shadow before and after tuning (reach 40 px, cubic falloff) |
| `05_glass_garden_motolii_bevy_three.png` | Glass Garden frames 0/120/240, Motolii row, Bevy row, three.js U72 S1/S4 |
| `06_final_posters_motolii_vs_bevy_stack.png` | final grid: chrome sphere, glass sphere, glossy plastic puck, extruded type × white/grey/red/black |

Against three.js U72, the gap is scene and image formation, not the material. three.js has a floor with reflections, light-cast shadows, tone mapping with bloom, and flower layers that see each other. Motolii still clips highlights hard and has no floor.

## Where each Bevy input comes from
| Bevy input | Motolii / re_renderer source | status |
|---|---|---|
| view and clip matrices | `frame.projection_from_world`, `view_from_world`, `camera_position` | provided |
| viewport, fragment position | derived from `projected_surface_uv(world_position)` × resolution | computed in the shader |
| exposure, frame count | none | constants 1.0 and 0 |
| diffuse environment map | `diffuse_shading` (same irradiance/π convention) | provided |
| specular environment map | `environment_specular_along`: equirect with plain box-filtered mips, not the GGX prefilter Bevy expects | provided, lower quality |
| DFG lookup texture | none | missing; Bevy's own polynomial fallback used |
| transmission texture | `backdrop_texture` (premultiplied, mipped) | provided; un-premultiplied before Bevy's blend |
| depth prepass | none | not needed: Motolii's backdrop holds only what is behind the layer |
| directional/clustered lights, shadow maps, ambient | Motolii's sun is the environment's brightest direction | unused; adding it would double-count the sun |
| ior, transmission, thickness, roughness, metallic, colour | Glass: Refraction, Transmission, instance scale, Roughness, Metallic, colour | mapped 1:1 |
| reflectance | computed from IOR | glue |
| attenuation, clearcoat, anisotropy, emissive | no Glass inputs | each needs a new effect input |
| dispersion | not in Bevy | glue: red and blue refract at slightly different IOR, as Motolii and three.js do |

Flattening Bevy's imports and shader switches was done by hand into one plain-WGSL module, with only the environment and specular-transmission paths turned on. Automatic WESL compilation isn't practical, as the T2 audit found. Bevy's transmission blur assumes metres and Motolii's world is in pixels, so the spike uses 1 m = 1000 px.

## Code size
| file | code lines | origin |
|---|---|---|
| `vism/material_bevy.wgsl`, Bevy block | 307, of which 16 edited (marked `// U76:`) | Bevy `ad31a06`, MIT OR Apache-2.0 |
| same file, glue | 78 (studio ≈ 40, input mapping ≈ 25, dispersion ≈ 6) | Motolii-specific |
| `vism/contact_shadow.wgsl` | 79 | Motolii-specific; blur filters are the ones already in `glow.wgsl` |
| `surface_program.rs` | +12 / −2 | env-var switch; Motolii's material stays loaded with its `shade_surface` renamed |

The material adds no GPU resources or passes. Contact Shadow adds three small float targets (1/4, 1/8, 1/16 size) and four fullscreen passes per layer that uses it, like Glow.

## Frame time (M4, 1080p, 60 moving frames, including readback)
| scene | Motolii | Bevy |
|---|---|---|
| Glass Garden | 55.3 / 57.1 ms | 59.4 / 59.5 ms (about +3 ms) |
| poster, no shadow | 4.2 / 4.3 ms | 4.2 / 4.2 ms |
| poster + Contact Shadow ×4 | 3.4 / 4.1 ms | 4.0 / 4.0 ms |

The poster numbers are mostly cache because the scene is static. Glass Garden misses the 16.7 ms target with either material, so the cost is elsewhere, not in the material.

## Limits
- **A bound environment replaces the composition's background colour** (`progress/02`). Once an environment image lights the scene, the background colour is gone. Fixing it means a host rule: the environment lights the scene but doesn't paint the backdrop. That is a semantic decision, so I stopped there; it needs your call or the user's.
- **Tone response:** highlights still clip hard (`saturate`), so there is no highlight roll-off. U77 has claimed this.
- **Specular prefilter:** box-filtered mips instead of a GGX prefilter. Not visible at poster scale.
- **Glass shadow:** Contact Shadow only sees alpha, so glass casts a dense shadow.
- **Emissive:** Motolii's material has no emissive; "emissive" today is lit colour plus Glow.
- **Clearcoat:** needs new Glass inputs; U77 has claimed it.
- **Harness bugs on this branch, not fixed** (they affect everyone's fixtures):
  - `.fill("#hex")` renders shapes white in `script_frames`. `linear-gradient(c, c)` works.
  - Extruded `text()` is offset by the comp centre, and `Tilt` throws it off-frame, so the type fixture is frontal.
  - Extruded text shows hatching artefacts inside letter holes.

## Controls to expose
- **Per object** (Glass as it is): Color, Roughness, Metallic, Transmission, Refraction (IOR), and Dispersion as an advanced control.
- **Per layer:** Contact Shadow → Strength.
- **Lighting:** none. The studio applies whenever no environment is bound; at most one preset choice.

Files are in `explore/U76/`:
- `wt/motolii/crates/motolii-render/vism/material_bevy.wgsl`
- `wt/motolii/crates/motolii-render/vism/contact_shadow.wgsl`
- `wt/motolii/crates/motolii-render/src/compositor/effects/surface_program.rs`
- `progress/`
- `NOTES.md`
- `scripts/`
- `render.sh`
- `sheet.sh`