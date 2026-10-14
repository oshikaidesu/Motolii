# Cavalry: the pictures people make, and what Motolii lacks for them (2026-10-14)

## Step 1: the words (written before searching)
Cavalry's own names: Duplicator; distributions Linear, Grid, Circle, Path, Point, Random, Shape/Sub-Mesh, Quad Tree, Voxelize, Sort/Shuffle/Mask; Stagger; Oscillator (with a Deformer tab); Noise; Random; Falloff (Circle, Rectangle, Linear, Sweep, Shape) and Range Falloff; Value / Value Array / Colour Array; Index to Color; Connect Shape; Points to Path; Get Vector; Look At; Sub-Mesh behaviour (per-character text); Trim; Path Offset; Spring; Forge Dynamics; Spreadsheet / Google Sheet; Sound behaviour; Math / JS Math; Dynamic Rendering; Gradient, Noise and Grain shaders.
Community words: kinetic typography, 36 Days of Type, stagger wave, ripple, grid animation, dot grid, halftone, op art / optical illusion, generative pattern / poster, seamless loop, plexus / connect lines, DNA helix, data viz / bar chart, falloff force / cursor push, isometric, grainy gradient, logo reveal.
(Kept at scratchpad/cavalry_words.txt.)

## Step 2: where I looked, and what I saw
- **The official YouTube channel:** I listed the latest 30 videos and three playlists with their view counts, including all 12 "1 Minute Wonders" and about 100 "Community Content" videos. For 35 videos I downloaded the thumbnail and YouTube's three automatic mid-video frames, 140 images in all, and looked at them on contact sheets (scratchpad/cav/sheets/s0-s6.jpg).
- **The official example files page** (cavalry.studio/docs/getting-started/example-files/): 14 demo scenes with the nodes each one uses.
- **Scenery (scenery.io):** the community gallery of shared Cavalry scene files. I downloaded all 244 posters that are not "36 Days of Type" and looked at every one (sheets/sc_part_00-05.jpg). Another 48 posters are 36 Days of Type letters.
- **Docs for Stagger, Falloff, Oscillator and Connect Shape,** to get the names of their knobs.
- **School of Motion articles,** read for the use-cases they name.
- **Not reached:** Instagram, Behance and Vimeo (the pages are not fetchable as text), and the forum. Scenery has no popularity sort, so the frequency counts below are my own eyeball tally of the 244 posters.

**Rough tally of the 244 Scenery posters:**

| Kind of picture | Posters |
|---|---|
| Animated type (plus 48 more "36 Days" letters) | ~45 |
| Op-art line and ring patterns | ~25 |
| Dot or shape grids pushed by noise or a falloff | ~20 |
| Radial bursts around a centre | ~12 |
| Isometric / extruded / fake 3D | ~15 |
| Gradient, blur and glitch shaders | ~15 |
| Connect-line plexus | ~10 |
| Character rigs | ~10 |
| Data and UI | ~6 |
| Physics | ~6 |

## What makes a picture "Cavalry"
Almost every iconic piece is one shape put through a Duplicator (a row, grid, circle or the letters of a text). One or two "drivers" then hand each copy a different value: a Stagger (evenly from min to max by index), an Oscillator (a sine with a per-copy phase), Noise (a field in space and time) or a Falloff (a shape you move; copies inside it change). The tutorials are short: the 1 Minute Wonders take 3–6 nodes and fewer than 10 knobs each. The look comes from four knobs:
1. the copy count and spacing;
2. the per-copy offset amount (the stagger spread, or the oscillator's "Stagger");
3. the driver's frequency or size;
4. a colour ramp by index.

Type is the biggest single topic. Cavalry splits a text into letters ("Sub-Mesh") and then treats the letters exactly like duplicator copies.

## Step 3: ranked list
Rank is by official tutorial views, presence in the official examples, and how often the picture repeats on Scenery.

**What Motolii is missing, by letter:**

| Letter | Missing piece |
|---|---|
| S | per-copy time delay (stagger) |
| N | values driven by noise or an oscillator |
| F | falloff field |
| C | connecting lines between copies |
| D | data-driven values |
| K | per-copy colour by index |
| L | text split into letters that act as copies |
| P | copies on a path, on random points or on a shape's points |

I added the last three (K, L, P) myself: they are outside your list, but the pictures need them.

1. **Wavy-line column (stagger wave).** Horizontal lines stacked in a column, each a wave with a different frequency, rippling. Recipe: Basic Line → Duplicator (linear, ~20–30 copies) → Oscillator used as a Deformer (Use Normals) → Stagger into "Number of Waves" (+Random). 4 nodes, ~6 knobs, 75 s video. Knobs: count, spacing, wave amplitude, stagger min/max of waves, speed. Needs **N S**. 58K views, the most-watched 1MW; it is also example "Infrequency". https://www.youtube.com/watch?v=4ZSKViGbdys — img https://i.ytimg.com/vi/4ZSKViGbdys/hq3.jpg ; https://scenery.b-cdn.net/scenes/pEHqGo9q8o1/poster.png

2. **Per-letter text ripple.** "RIPPLE" in yellow on pink; the letters rise and fall one after another. Recipe: Text Shape → Sub-Mesh deformer → Stagger (or Random) on each letter's time offset or position; the key is a single up-down. 3 nodes, ~4 knobs, 65 s. Knobs: stagger spread in frames, rise height, ease. Needs **L S**. 40K views. Repeats on Scenery as type-ripple, wave-type, wavetext, falloff-type and most of the 48 36-Days letters. https://www.youtube.com/watch?v=T9S7fDMcfOI — img https://i.ytimg.com/vi/T9S7fDMcfOI/hq3.jpg

3. **Dot grid pushed by a cursor (falloff force).** A grid of red dots; dots near a moving pointer are pushed away into a ring-shaped hole. Recipe: Circle → Duplicator (grid, ~12×12) → Falloff (Circle) driving a position push away from the centre; a "Value" behaviour holds the push amount. ~4 nodes, ~6 knobs. Knobs: grid count and spacing, falloff radius, falloff curve, push strength. Needs **F**. 13K views; the 1MW "Using Falloffs" (17K) has the same core. https://www.youtube.com/watch?v=1-bjB-xSuYw — img https://i.ytimg.com/vi/1-bjB-xSuYw/hq1.jpg ; https://www.youtube.com/watch?v=WX54t_UKnOE

4. **Op-art illusion.** A black-and-white zig-zag starburst of nested shapes, turning against each other. Recipe: shape → Duplicator (nested, scaled copies) → Oscillator on rotation with per-copy stagger. 2–3 nodes. Knobs: count, scale step, rotation amplitude, the oscillator's stagger. Needs **N S**. Views: 14K plus the "Optical Art" example. Scenery repeats it as osci-art, black-white-waves, stars-and-stripes, unknown-pleasures and r-series-001-oparthills. https://www.youtube.com/watch?v=qemV44m3iUY — img https://i.ytimg.com/vi/qemV44m3iUY/hqdefault.jpg

5. **Iris / radial line burst.** Short lines around a centre forming a ring, coloured pink, purple and blue by index. Recipe: Line → Duplicator (Circle distribution) → Index to Color. 3 nodes, 68 s. Knobs: count, radius, line length, colour ramp. Needs **K**; it is static, and the rotation and size already exist. 15K views. https://www.youtube.com/watch?v=vB9MQ_MKp8s — img https://i.ytimg.com/vi/vB9MQ_MKp8s/hqdefault.jpg

6. **Halftone spiral.** Yellow dots on a ring/spiral, large at the middle of the band and shrinking outward. Recipe: Duplicator (circle/spiral) → Falloffs driving scale. 2–3 nodes. Knobs: count, spiral turns, falloff size and curve. Needs **F**, and **P** for the spiral. 16K views. https://www.youtube.com/watch?v=yHUUCLe2zP0 — img https://i.ytimg.com/vi/yHUUCLe2zP0/hqdefault.jpg

7. **Noise dot grid.** A grid of white dots whose sizes bloom in noisy blobs that drift. Recipe: Circle → Duplicator (grid) → Noise on scale. 3 nodes, ~5 knobs. Knobs: grid size, noise scale, speed, min/max size. Needs **N**. Related: 1MW "colourful looping background", 14K: colour bands from Duplicator + Noise + Math. https://scenery.b-cdn.net/scenes/kcakjQ1ZgW5/poster.png ; https://www.youtube.com/watch?v=Nn38fEGdiLk — img https://i.ytimg.com/vi/Nn38fEGdiLk/hqdefault.jpg

8. **Plexus / connect lines.** Letters or dots pushed away from a moving point, joined by thin lines between near neighbours. Recipe: Text Shape → Sub-Mesh → Get Vector (push) → Connect Shape (Within Range, Search Distance, Per-Point Limit). 4 nodes. Knobs: search distance, connection limit, push strength. Needs **C F L**. 18K views. Scenery: connect-shape, connect-noise, r-series-005 spider, forge-with-connect-shape. https://www.youtube.com/watch?v=pMGuLcln4SQ — img https://i.ytimg.com/vi/pMGuLcln4SQ/hqdefault.jpg ; https://scenery.b-cdn.net/scenes/nmEljGx6dXd/poster.png

9. **DNA helix.** Two columns of coloured dots swinging in opposite phase, joined by rungs. Recipe: Duplicator (linear) → Oscillator on x with stagger → Value Array colours → Points to Path for the rungs. 4 nodes, 2.5 min. Knobs: count, amplitude, phase stagger, palette. Needs **N S C K**. 15K views; "DNA style text" adds 14K. https://www.youtube.com/watch?v=pLr0CAoZnXM — img https://i.ytimg.com/vi/pLr0CAoZnXM/hqdefault.jpg

10. **Data bar chart.** Pink card, "+82.35%", bars growing to values from a Google Sheet, with labels. Recipe: Rectangle + Text → Duplicator (linear) → Spreadsheet → Number Range → Bounding Box constraint → Index to Color → Contrasting Color → random start. ~8 nodes, 22 min. Knobs: data column, value remap, bar gap, colours. Needs **D K S**. 11K views (+9.7K for a population map: thousands of dots placed and sized by a sheet). https://www.youtube.com/watch?v=--1_TJJjJ4U — img https://i.ytimg.com/vi/--1_TJJjJ4U/hqdefault.jpg

11. **Generative identity posters.** Quad-tree cells filled with random arcs, rings, stripes and "BIG POSTER" type; 100 variants are rendered from seeds. Recipe: Quad Tree Shape → Sub-Mesh distribution → Random shapes and colours → Dynamic Rendering. ~6 nodes, 19 min. Knobs: subdivision depth, seed, palette. Needs **D P K** (random choice per cell). 17K views; the abstract-artwork video adds 28K. https://www.youtube.com/watch?v=qRTFaOzKFvs — img https://i.ytimg.com/vi/qRTFaOzKFvs/hqdefault.jpg

12. **Ring illustration.** A swirl of crescents (two circles masked) around a centre. Recipe: two Ellipses + Mask → Duplicator (circle). 3 nodes, 70 s. Knobs: count, radius, crescent offset. Needs nothing new beyond the mask. https://www.youtube.com/watch?v=4N2YCuzxS2U — img https://i.ytimg.com/vi/4N2YCuzxS2U/hq3.jpg

13. **Concentric oscillating rings** ("Concentrick"). Rings with staggered radii whose centres sway. Recipe: Circle → Duplicator → Stagger on radius → Oscillator on position. Needs **N S**. https://scenery.b-cdn.net/scenes/bKzaa2YsAMq/poster.png

14. **Rotating-line maze** ("Mazin"). A grid of short diagonal lines flipping between two angles as a falloff passes, so the pattern looks like a maze. Recipe: Line → Duplicator (grid) → Value Array (2 angles) → Falloff → JS Math. Needs **F**. https://scenery.b-cdn.net/scenes/A80p6r9DrqC/poster.png

15. **Ring with a noisy wobble** ("Ring Ting"). Dots on a circle; a falloff blob lets noise push the dots near it. Recipe: Duplicator (path distribution) → Noise → Falloff. Needs **N F P**. https://scenery.b-cdn.net/scenes/GRUG07vs3gJ/poster.png

16. **Depth-of-field grid** ("Focus"). A 4×4 grid of glossy green balls, each blurred by its distance from a focus point. Recipe: Circle + gradient + blur with the amount from a falloff or oscillator. Needs **F N**. https://scenery.b-cdn.net/scenes/J9JaB3umkHv/poster.png

17. **Physics collisions / falling type.** Circles bump each other; letters drop onto a floor and pile up. Recipe: Forge Dynamics + Noise (Pro). Knobs: gravity, bounce, start offsets. Needs physics (outside the list). Views: 22K, plus falling type 19K and fluid type 20K. https://www.youtube.com/watch?v=bWmz-R7UhNs ; https://www.youtube.com/watch?v=bVBwnsi2M_c — img https://i.ytimg.com/vi/bVBwnsi2M_c/hqdefault.jpg

18. **Audio equaliser.** Bars jumping to music. Recipe: Sound behaviour → Duplicator → Align. 3 nodes. Needs **D** (audio as data) **S**. 18K views. https://www.youtube.com/watch?v=OT9qvHYEnmw

19. **Picture-as-grid / animated pixel grid.** A skull drawn as a grid of minesweeper tiles that switch over time. Recipe: Duplicator (grid) driven by image luminance. Needs **D F**. 11K views. https://www.youtube.com/watch?v=6RXVESNx7hc — img https://i.ytimg.com/vi/6RXVESNx7hc/hqdefault.jpg

20. **Boolean radial pattern.** An orange square with black lines and petals swirling around a centre. Recipe: Duplicator (circle) + Boolean + rotation. Needs mostly existing pieces plus a boolean. 37K views. https://www.youtube.com/watch?v=VOT_Rm0x6SQ — img https://i.ytimg.com/vi/VOT_Rm0x6SQ/hq3.jpg

21. **Interlocking arc tiles** ("Interlink"). Wavy arcs in a grid on green, rotating with looping offsets. Recipe: Arc → two nested Duplicators → Value Array → offset curve. Needs **S**. https://scenery.b-cdn.net/scenes/2k3gXRKjXRg/poster.png

22. **Gradient along a path.** A thick line wiggling, with a rainbow running along it. Recipe: Line → Subdivide → Oscillator deformer; Gradient stroke mode. Needs **N** and a gradient along the stroke. https://www.youtube.com/watch?v=CIvm5H0-1Mg — img https://i.ytimg.com/vi/CIvm5H0-1Mg/hqdefault.jpg

## What this says for Motolii
- **S and N cover the most pictures.** A per-copy delay (S, the Stagger) plus a per-copy sine or noise value (N) is needed by 1, 2, 4, 7, 9, 13 and 21, the core of Cavalry's look. Cavalry's Oscillator itself has a "Stagger" knob, so N and S are often one control.
- **F is next.** A falloff, shaped as a circle, rectangle, linear band or sweep, that a moving thing carries is needed by 3, 6, 14, 15 and 16.
- **K is cheap and visible everywhere.** Per-copy colour from a palette ramp is in 5, 9 and 10 and on most Scenery posters.
- **L is the gate to type.** Letters as copies unlock the single biggest topic, about a fifth of Scenery.
- **C and D are real but rarer:** about 10 plexus pieces and about 6 data pieces among 244.
