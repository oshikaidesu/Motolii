# Silhouette critique: Worlds A-E (29 small worlds)

Judge: independent, running Widgetbook (lab_book), Palette B, 2026-10-02. No code touched.
Scope note: the book holds 29 worlds, not 30 (D has 19, 21-24; #20 Transform is absent). World C is two use cases per world (large + row); A, B, D, E show large + 282 px row together.
Method: full-res screenshot per world (Driven Off), then one drag on a grab zone (zoom crop after), Driven = Pulse on the first world of each set plus Stagger, Rotation, Glass (A-set Falloff and Stagger, B Rotation, C Blur, D Glass, E Easing). Words Hidden on 10: Scale, Falloff, Direction, Scatter, Stagger, Rotation, Repeat, Noise, Graph, Audio (plus Blur and Glass glanced).
Caveats (honest): idle life is judged from two or three frames per world, not a recording; "idle" = Y only where I saw the graphic change with Driven Off (Echo comet, Wave Warp flow, Noise scroll, Audio arcs, Falloff/Direction drift unconfirmed). Pulse was not run on every world (not run: Time remap, Camera, Wiggle, Loop, Glow, etc.). Hover-only-highlights was seen as state change on the pointer target only; no world moved geometry on hover that I noticed.
Screenshots: default shots copied to `/Users/member_ottoto/rust_ae/design-sense-lab/research/shots/world-<id>-<name>.jpg` (full app window, 29 files). Originals under `/Users/member_ottoto/.claude/projects/-Users-member-ottoto-rust-ae-Motolii/77f277f7-741f-4e9f-8e94-ddd760f7f088/tool-results/mcp-computer-use-blob-<ts>-<id>.jpg`; ids in the last column (default / Pulse or drag crop where taken).

Columns: Sil = silhouette phrase (FAIL if slider/knob/dial/XY/bar/stepper/handle/number box); World = dedicated small world expressing the concept; Zones = 3-4 grab zones colour-matched to elements (A blue, B green, C white, D orange); Move = graphic itself moves under Driven (modulation visible); Idle = subtle idle life seen; N = number is a quiet readout; Tone = calm grey / 1px / fine; WH = purpose clear with Words Hidden (- not tested); Row = 282 px row readable and unclipped.

| # | Name | Sil | World | Zones | Move | Idle | N | Tone | WH | Row | Defects | Verdict | shot ids |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| A1 | Falloff | ripples round a falling drop | Y | Y (blue ticks radius, green arcs curve, white drop strength, orange centre) | Y (all four, rings shift/clip) | ? | Y (coloured legend dots) | Y | Y | Y | rings clip at box edge when centre is pulled to a corner (natural) | KEEP | a6bsio / a8ebej, h1zoh9 (pulse) |
| A2 | Direction / Attract | grass leaning toward a pin | Y | Y | - | ? | Y | Y (dense, fine) | Y | Y (arrows run to the border) | none; target drag curls whole field, convincing | KEEP | 10u3ml / 028abp |
| A3 | Scatter | a few seeds thrown on the ground | Y | Y (blue ticks spread) | - | ? | Y | Y | Y (weak) | Y | content uses ~100 px of a 310 px box, 20 dots, reads as sparse specks; drag 60 to 133 works | FIX (scale up the cloud) | 21jod7 / n7ak5h |
| A4 | Stagger | cascade of cards beside a walking line | Y | Y (dots on first/mid/last card) | Y (pulse re-fans cards, rotates line) | N (static fan) | Y | Y | Y | Y | four drags (cards, dots, line) changed no value; grab dots ~5 px; the line only repositions | FIX (zones unreachable) | ppthb6 / ercr45, 755prg |
| A5 | Along Path | necklace between two pennants | Y | Y (blue/green flags, white bead, orange ring) | - | ? | Y | Y | - | Y | none; start flag slid 0 to 31 % and beads re-spaced | KEEP (a best world) | 45dxjr / 9jglif |
| A6 | Scale distribution | stream of soap bubbles | Y | Y | - | ? | Y | Y | Y | N | min 40 to 200 % saturates and bubbles overflow the left edge of the box and row | KEEP (clamp) | 0nv8fe / mhvk5e |
| B1 | Rotation distribution | fan of pinned needles | Y | Y (blue first, green last, orange pivot) | Y | ? | Y | Y | Y | Y | step 20 to 29 and jitter cross the needles into a scrawl; pulse destroys the fan | FIX (clamp step, cut jitter crossing) | yhrbwr / 4337gh, noi6t6 |
| B2 | Colour gradient | beads on a wire | Y | Y (blue/green end rings, white tick, orange ring) | - | ? | Y | Y | - | Y | hue drag along the wire works; beads on a wire can read as a range strip | KEEP | mxmq42 / yn9kxt |
| B3 | Repeat | hex lattice with ghost slots | Y | Y (blue ghost column, green ghost row) | - | ? | Y | Y | Y | Y | none; 5x3 to 6x3 with ghosts shifting | KEEP (a best world) | 9h3bfd / 1bcw2w |
| B4 | Noise | seismograph strip with a loupe | Y | Y (blue loupe, orange pen, white dashes) | - | Y (trace scrolls) | Y | Y | Y | Y | pen drag no effect; white bracket dashes are tiny plain handles; reads as a line chart | FIX | 3hqx6h / lpa668, ahxzd4 |
| B5 | Graph / Link | two beads on a bezier thread | N-ish (easing-curve editor look) | Y | - | N | Y | Y | N | Y | strength .70 to 1.00 barely visible; type diamond drag no effect; empty lower half | FIX (or fold into Easing) | 6s1vcp / as8k1y, b3evjy |
| B6 | Audio react | ear, gate arc and a shaken bead | Y | Y (orange ear, green bead ring) | Y (arcs pulse past the gate) | Y | Y | Y | Y | Y | at rest nearly blank, ear icon ~12 px; threshold drag was the trigger for the arcs | KEEP | 3eiyjd / 28ygji |
| C1 | Gaussian Blur | hexagon dragging a smear | Y | Y (blue rim, green vertices, white tick, orange cross) | Y (radius, vertex count 6 to 7, smear) | ? | Y | Y | N | Y (tiny) | graphic ~25 % of the box; vertex/centre drags did nothing, only the tick handle works; looks like a badge when Words hidden | FIX (scale up) | tl9icq / 4vsjlm, j1ckm4 |
| C2 | Glow | star over a dashed threshold line | Y | Y (blue radius brackets, dashed white threshold, orange ray ticks) | - | ? | Y | Y | - | Y | radius 40 to 183 px on a modest drag (sensitive) | KEEP | v3dbq4 / 4nq75s |
| C3 | Echo | comet on a figure-eight with linked beads | Y | Y (blue tail ring, orange head) | - | Y (comet runs) | Y | Y | - | Y (readout touches path) | link outlines between beads read as a robot-arm linkage | KEEP | de53bz / sou0da |
| C4 | Wave Warp | rippled grid, a flag | Y | Y | - | Y (waves flow) | Y | Y | - | Y | grab dots ride the wave so drags miss; wavelength drag works | KEEP | 9jngkb / hnjcdp |
| C5 | Fractal Noise | contour map | Y | ? (blue dot drag changed Complexity, not Scale) | - | ? | Y | Y (busy) | - | Y (readout on lines) | possible zone/colour mismatch | KEEP (check mapping) | cqirsq / vgxefw |
| C6 | Shadow | block, hatched shadow and sun on grey ground | Y | Y (sun green, shadow blue, orange hatch) | - | ? | Y | N (box is flat lit grey) | - | Y (readout touches the block) | whole box filled grey, unlike every other world; small objects; angle 135 to 237 flips shadow correctly | FIX (calm the ground) | ejthz7 / qbqvsp |
| D1 | Glass | slanted glass wedge bending a beam | Y | Y (blue refracted ray) | Y (dispersion fan shimmers) | N (slab static) | Y (mono, no colour dots) | Y | Y | Y | thickness drag saturates at 100 px; readout carries no slot colours | KEEP | drqfqf / zq7rw0 |
| D2 | Opacity / Blend | two frosted plates | Y | N | - | N | Y | N (large flat greys) | - | Y | brightest flat fills, no detail; blend/clip/ghost zones invisible | DROP (superseded by the cat behind a misted pane) | np4wq8 / zlcqew |
| D3 | Fill | pink pigment pebbles on dotted paper | Y | N (blue and orange specks only) | - | ? | Y | N (saturated pink) | - | Y | loudest element after D2; hue drag works | FIX (quieter, cut to one pebble + hairline) | o6a3em / 4clry8 |
| D4 | Trim paths | knotted line being drawn | Y | N (end marks barely visible) | - | ? | Y | Y at rest, N dragged | - | Y (line hugs edges) | tip drag changed Width 2 to 60 px = fat white blob | FIX (clamp, show ends) | dgapfi / gpv728 |
| D5 | Mask feather | dotted window with fading rim contours | Y | Y (blue rim, orange centre) | - | ? | Y | Y | - | Y | none; feather 0 to 64 px adds dotted concentric rims | KEEP (a best world) | g1maif / viy7k8 |
| E1 | Easing | footprints along a ribbon | Y | Y (blue dots In, green Out, orange runner) | Y (spacing changes) | ? | Y | Y | - | Y | In 33 to 98 % is a big jump; plain curve with dots, not a bezier handle | KEEP | xcie4d / gg2akc, b3svq3 |
| E2 | Spring | weight on a coil with an ink trail | Y | Y (blue coil, green damper, orange rest) | - | ? | Y | Y | - | Y | weight drag changes Mass and the trail bounces; good | KEEP | 3433go / ctf78w |
| E3 | Wiggle | a stray dot in dust | Y | Y | - | ? | Y | Y | - | Y | graphic tiny in the box; amplitude jumped 20 to 200 and a ring bigger than the box was drawn | FIX (scale, clamp) | 1lzjw2 / t1dbb5 |
| E4 | Time remap | a loom of threads | Y | Y | - | ? | Y | Y | - | Y | none; middle thread drag straightens threads (Smooth 38 %) | KEEP (a best world) | 6menk2 / 8ij85q |
| E5 | Loop | racetrack ellipse with ripple rings | borderline (ring + dot is still dial-like) | Y | - | ? | Y | N (moire blue rings) | - | Y | blue concentric rings loud; dot on ring = dial | FIX | 3386g5 / 3kojsp |
| E6 | Camera | cone seen from above with a viewfinder | Y | Y (blue cone, green arc, orange frame) | - | ? | Y | Y | - | Y | subject drag changed no readout | KEEP | b22qpx / khce64 |


## Totals
- KEEP 17, FIX 11, DROP 1 (D2). Silhouette fails or at risk: B5 Graph/Link (easing-editor look), E5 Loop (dial), B2 Colour (range strip risk, mild).
- Words Hidden (10 tested): clear for Scale, Falloff, Direction, Stagger, Rotation, Repeat, Noise, Audio; weak for Scatter; not clear for Graph; also not clear for Gaussian Blur. No text leak: Hidden removes caption and readout, leaving four coloured legend dots (blue/green/white/orange). That fixes the Phenomenon set C caption leak.
- Idle: confirmed only for Echo, Wave Warp, Noise, Audio; static-looking: Stagger, Glass slab, Rotation, Opacity.

## Top defects
1. Scale: A3 Scatter, C1 Blur, E3 Wiggle use a quarter of the box; A6 and D3 overflow; fix with content-to-box fitting.
2. Hit zones: A4 Stagger (4 drags dead), B4 Noise pen, B5 diamond, C1 vertices, E6 subject; zones are 5 px or ride moving elements.
3. Sensitivity: D4 width 2 to 60, E3 amplitude 20 to 200, A6 min to 200 %, D1 thickness to 100, C2 radius 40 to 183, E1 In 33 to 98 within one short drag.
4. Loud: D2 grey plates, D3 pink pebbles, C6 grey ground, E5 blue moire, D4 dragged line.
5. Number readouts differ: D1 Glass readout has no slot colours; others use coloured legend dots (A set best).
6. C5 blue dot changed Complexity (D slot) rather than Scale (A).

## Phenomenon vs World (tournament input)
- Scatter: Phenomenon A1 flock+ring is livelier, World A3 is controllable and has colour legend but only ~20 dots in a quarter of the box. Winner: Phenomenon A1 (fix its sensitivity, borrow the World legend).
- Stagger: Phenomenon A2 fanned cards with working drag beats World A4 (equal look, zones dead); World A4's walking line and pulse are nicer. Winner: A2, borrow the line.
- Falloff: World A1 ripples beat Phenomenon A5 (A5 clips, slider-ish line; A1 moves on all four under Pulse). Winner: World A1.
- Glow: World C2 (star + threshold line + sources) beats Phenomenon B1 (star alone). Winner: World C2.
- Echo: Phenomenon E4 comet on figure-eight is cleaner than World C3 (linked outlines read as linkage). Winner: Phenomenon E4.
- Warp: Phenomenon C3 rubber grid with a grabbable bulge is the stronger toy; World C4 flag is the calmer picture of Wave Warp (a different concept). Winner: C3 for Warp; C4 for Wave.
- Noise: World C5 contour map beats Phenomenon C2 ridge; World B4 seismograph is a line chart (weaker than C2). Winner: World C5 for property noise.
- Blur: World C1 adds the smear tail and aperture vertex count but is tiny; Phenomenon B5 bare hexagon is idle-dead. Winner: World C1 once enlarged (B4 DoF cards stay separate).
- Shadow: Phenomenon B3 (dotted floor, sun on tether) beats World C6 (flat grey ground). Winner: Phenomenon B3.
- Easing: World E1 footprints beats Phenomenon D1 (pen-tool bezier silhouette is the precedent, not the toy). Winner: World E1.
- Spring: World E2 (weight drag changes Mass, bouncing trail) beats Phenomenon D2 (gauge-like tube). Winner: World E2.
- Wiggle: Phenomenon D3 jittery thread beats World E3 (tiny, drag saturates). Winner: Phenomenon D3.
- Time remap: World E4 loom beats Phenomenon D5 belt (bar). Winner: World E4.
- Loop: World E5 racetrack is better than D4 ring and dot but still dial-like. Winner: World E5 after calming the rings.
- Camera shake vs Camera: different concepts; Phenomenon D6 owns shake, World E6 owns lens/frustum.
- Mask feather: World D5 dotted window beats Phenomenon B6 pebble. Winner: World D5.
- Stroke: World D4 knotted line beats Phenomenon C5 thick wavy line once the width is clamped. Winner: World D4.
- Opacity: Phenomenon C6 cat behind misted pane beats World D2 plates by far. Winner: Phenomenon C6.
- Repeat: World B3 hex lattice with colour ghosts beats Phenomenon A3 file of tiles. Winner: World B3.
- Glass/Refraction: World D1 adds dispersion and thickness; Phenomenon B2 clearer single ray. Winner: World D1 (give its readout slot colours).
- Rotation fan vs Stagger fan: keep Rotation = needles, Stagger = cards (as op1-translation recommends).
