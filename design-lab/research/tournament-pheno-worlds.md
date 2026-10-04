# Re-judge and tournament: Phenomenon vs World

Judge: independent, running Widgetbook (lab_book), Palette B, Words Full, 2026-10-02. No code touched.
Method: per item one full-resolution default screenshot, one app_drag + 0.6-scale screenshot; Driven = Pulse run on worlds E3 Wiggle, E5 Loop, E6 Camera, D1 Glass, D4 Trim, B1 Rotation (not run on the others, time). Words Hidden not re-tested this round.
Blobs: `/Users/member_ottoto/.claude/projects/-Users-member-ottoto-rust-ae-Motolii/77f277f7-741f-4e9f-8e94-ddd760f7f088/tool-results/mcp-computer-use-blob-<id>.jpg`. Ids below are the full `<ts>-<id>` part of the blob file name.
Caveat: Falloff dots, Wiggle dust, Loop frames and Time-remap ball motion were judged from 2-3 frames, not a recording.

## 1. Re-judge: Phenomenon

| # | Name | Silhouette | Prev defect | New defect | Verdict | default shot / drag shot |
|---|---|---|---|---|---|---|
| A1 | Scatter | loose flock of dots round a hollow heart, bigger die icon | drag 36->1: FIXED (drag now draws a pink spread ellipse, count stays 36); row merge: FIXED (dots separate) | none | KEEP | 1790917911848-wvvmi5 / 1790917915613-dake6i |
| A4 | Grid | tiled frame of rounded cells with a corner bracket on cell 1 | static: PARTLY (tiny breathing dot in each cell, first-cell bracket) | still reads as a layout wireframe / table; life is faint; drag works (4c3r -> 3c2r) | FIX (leaning DROP, Repeat covers it) | 1790917898378-grk4af / 1790917902326-v8hy74 |
| A5 | Falloff | dot field with concentric region rings and four larger shape icons | clip, 10px icons, slider-ish line: FIXED (line gone, rings, icons ~16px, row unclipped) | ring + dots reads slightly like a radar/target; size drag 140 -> 294px for a 41px move | KEEP | 1790917889595-bixy4j / 1790917893585-prdke4 |
| B5 | Blur bokeh | translucent hexagon smeared into a streak with bokeh specks | bare-hexagon idle: FIXED (smear stack and specks at rest) | hexagon is the brightest thing; fine | KEEP | 1790917870779-n5nqf5 / 1790917874261-hhyan9 |
| B6 | Mask feather | pale pebble with a rim grab ring, inner contour when pushed | loud flat fill: FIXED (dim fill, hairline contours); rim rub visible: FIXED | blob still unclear without words | KEEP (weak) | 1790917878755-fmh7wf / 1790917882436-gkhqlw |
| C5 | Stroke | a pen path with its thickness ribbon and node dots | thick white line: FIXED (hairline ribbon); collapse 12->1px: FIXED (12->15px for 28px) | reads as a bezier path being edited (pen-tool precedent), little life | KEEP (weak) | 1790917862315-7a3hdq / 1790917866015-75vg48 |
| D2 | Spring | coil and orange ball in a rounded vial of dust, ring-down trace beside | tube ticks / gauge: FIXED (ticks gone, vial with dust) | none; ball drag launches a ring-down trace | KEEP | 1790917837100-fera6s / 1790917840668-2hzjii |
| D4 | Loop | racetrack ring of tiny film frames, a runner and a seam | dial silhouette: FIXED (frames, not a dial) | small in the 320 box; drag on seam switched mode to ping | KEEP | 1790917844759-0co3vn / 1790917848704-d7srvl |
| D5 | Time remap | draped strip of film frames with a ball in each, bracket viewfinder | long ticked bar: FIXED (curved drape) | none; drag 1.00x -> 2.20x compresses the strip | KEEP | 1790917852542-kifadl / 1790917855843-n8m1qi |
| E1 | Macro | small round face with blush; three readouts appear only while dragging | 8 numbers and row clip: FIXED (3 quiet readouts glow/blur/shrp, no clip) | none | KEEP | 1790917808776-7z68rb / 1790917813410-kg91va |
| E2 | Variations | one big face beside a 4x2 shelf of larger faces on two ruled rows, linked from/to rings | tiny hit zones: FIXED (faces ~45px) | none; drag rewires from/to and mix 100% | KEEP (a best toy) | 1790917817752-6x8vra / 1790917821653-gbm82j |
| E3 | React-to | two orbs with arcs joined by a sagging wire, dashed bounds | curve past scale: FIXED (bounded dashed envelope; +40 -> +87 for 41px) | none | KEEP | 1790917825869-8u7btd / 1790917829403-a1s689 |

## 2. Re-judge: Worlds

| # | Name | Silhouette | Prev defect | New defect | Verdict | default / drag (/ pulse) |
|---|---|---|---|---|---|---|
| A3 | Scatter | seeds thrown on the ground, coloured ticks | tiny cloud: FIXED (spans ~250px of 310) | still only ~40 dots; fine | KEEP | 1790918082542-8lmu7e / 1790918086343-23w6r3 |
| A4 | Stagger | cascade of cards beside a walking line | dead zones: NOT FIXED (readout stays 80ms/100/30/1.00; only the line repositions; dots ~5px) | static | FIX | 1790918089926-3j3vf9 / 1790918093974-eary06 |
| A6 | Scale distribution | stream of soap bubbles, tiny ripple arcs | overflow: FIXED (stays inside box and row) | min 40 -> 184% for a 35px drag (sensitive but bounded) | KEEP | 1790918100086-y7r2v7 / 1790918103585-sb2dzo |
| B1 | Rotation distribution | fan of pinned needles | scrawl: FIXED (step clamps, jitter no longer crosses; Pulse keeps a clean fan) | step 0 gives parallel needles (fader-bank look, natural) | KEEP | 1790918049211-2a9028 / 1790918052995-txxm1o, pulse 1790918058807-8220f4 |
| B4 | Noise | waveform area with ghost ridges and a loupe | line chart: NOT FIXED | pen drag now works (seed 3 -> 8) but silhouette is still an audio-wave chart | DROP (World C5 is the noise toy) | 1790918062915-b7376a / 1790918066558-b54fxh |
| B5 | Graph / Link | two beads on a slack thread, lag tick | easing-editor look: PARTLY (now a hanging thread, not a bezier editor) | middle handle drag changed nothing (slack stays .50); empty box; reads as a node-graph connector | FIX | 1790918070530-03x4z8 / 1790918074960-ron1ro |
| C1 | Gaussian Blur | hexagon with ring stack, orange cross | tiny: NOT FIXED (~70px in 310) | vertex drag now works (6 -> 8 gon); with Words Hidden still a badge | FIX (scale up) | 1790918014796-10s9lv / 1790918018761-18jtx1 |
| C2 | Glow | star over a dashed threshold line, dots above it ring and glow | n/a (KEEP) | radius 40 -> 81 for 20px (sensitive) | KEEP (a best world) | 1790918038335-ot31lw / 1790918042281-lf7g0i |
| C5 | Fractal Noise | contour map, dotted, with slot-coloured specks | zone mismatch: NOT FIXED (blue dot drag changed the map but the readout stayed 200px/100%/0/3) | none visual | KEEP (check mapping) | 1790918022787-amak32 / 1790918026506-hhav0n |
| C6 | Shadow | block, hatched shadow, sun, on dark dotted ground | flat grey ground: FIXED | block ~55px in 310, small; sun drag works (135 -> 252, 10 -> 18) | KEEP | 1790918031195-tsltyf / 1790918034778-dpqj60 |
| D1 | Glass | slanted slab bending a beam, dispersion fan | readout slot colours: FIXED (blue/green/white/orange dots) | thickness 20 -> 31 for 30px, no longer saturates; Pulse moves ior/rough/thickness/dispersion | KEEP (a best world) | 1790917996063-mpat2m / 1790917999861-uhah5r, pulse 1790918005316-wvsl7r |
| D2 | Opacity / Blend (not deleted: now a cat behind two frosted panes) | cat behind a hatched pane with a second outlined pane | flat grey plates: FIXED | duplicates Phenomenon C6; blend drag worked (Normal -> Screen) | KEEP (duplicate, lose to C6) | 1790917988242-03wa9v / 1790917992177-ib952l |
| D3 | Fill | translucent pigment strata on dotted paper with corner marks | loud pink: FIXED (muted, translucent) | hue drag shifts #C95E8B to #C95E5E (warm brick); fine | KEEP | 1790917981111-x3if7f / 1790917984645-que7n9 |
| D4 | Trim paths | knotted fish-line; ends appear when trimmed | clamp: FIXED (width no longer blobs); end marks: PARTLY (hidden at 0-100%, visible under Pulse) | tip drag at rest changed nothing (ends are at the path start, not on the crossing) | KEEP (end marks at rest) | 1790917966178-e2t83d / 1790917970112-aqo938, pulse 1790917976083-rcvwvz |
| E3 | Wiggle | stray dot in a dust cloud with an amplitude ellipse | tiny/ring over box: FIXED (cloud wide; amp 20 -> 54 bounded ellipse inside box) | in the 282 row cloud is still small | KEEP | 1790917924480-ymj2zc / 1790917928413-h548oi, pulse 1790917936619-up7f09 |
| E5 | Loop | racetrack ring of film frames, runner, loop strip inside | blue moire: FIXED (frames); dial: FIXED | none; Pulse changes count 11, start 6%, end 97% | KEEP | 1790917941938-q7cpgf / 1790917945996-0b16ka, pulse 1790917950836-p1020o |
| E6 | Camera | cone seen from above, orbit arc, viewfinder frame | subject drag dead: FIXED (distance 1000 -> 469px, camera moves) | none; Pulse moves all four | KEEP | 1790917954268-ljecoa / 1790917957417-e5sbhw, pulse 1790917962652-7skt3h |

Re-judge totals: Phenomenon KEEP 11 (B6 and C5 weak), FIX 1 (A4), DROP 0. Worlds KEEP 13, FIX 3 (A4, B5, C1), DROP 1 (B4).

## 3. Tournament

Criteria: silhouette is the phenomenon, number a quiet 4th element, calm with fine detail, small physics life, every grab zone responds, works in the 282 px row, understandable with Words Hidden (Hidden not re-tested; carried from the earlier critique).

| Concept | Pheno | World | Winner | One reason | Winner shot |
|---|---|---|---|---|---|
| Scatter | A1 flock + hollow heart + die | A3 seeds + coloured ticks | Phenomenon A1 | calmest silhouette, a drag draws the spread ellipse and nothing jumps; A3 ticks are control-like | win-scatter.jpg |
| Stagger | A2 fanned cards on a hinge | A4 cascade + walking line | Phenomenon A2 | all drags work; World A4 zones still dead | win-stagger.jpg (earlier build) |
| Falloff | A5 target rings + shape icons | A1 ripples round a falling drop | World A1 | ripples are the phenomenon, A5 keeps four icons and a target look | win-falloff.jpg (earlier build) |
| Glow | B1 star in halo | C2 star over threshold, dots above ring | World C2 | threshold dots show what glows; star alone is a lamp | win-glow.jpg |
| Echo | E4 comet on a figure-eight | C3 comet with linked beads | Phenomenon E4 | pure comet life; World linkage outlines read as a robot arm | win-echo.jpg (earlier build) |
| Warp | C3 rubber grid with a bulge | C4 flag-wave (Wave Warp is another concept) | Phenomenon C3 | grabbable bulge in a calm grid | win-warp.jpg (earlier build) |
| Noise | C2 ridge line | C5 contour map (B4 seismograph dropped) | World C5 | a contour map is not a line chart; fine, dense, calm | win-noise.jpg |
| Blur | B5 smeared hexagon + bokeh specks | C1 hexagon rings, tiny | Phenomenon B5 | fixed idle, readable at size and in the row; C1 is a 25% badge | win-blur.jpg |
| Shadow | B3 block, floor, sun on tether | C6 block, hatch, sun on dark ground | Phenomenon B3 | larger objects and the tether; C6 is correct but tiny | win-shadow.jpg (earlier build) |
| Easing | D1 bezier + hop arc | E1 footprints along a ribbon | World E1 | footprints are the phenomenon; D1 is the pen-tool precedent | win-easing.jpg (earlier build) |
| Spring | D2 coil in vial, ring-down trace | E2 weight on coil with ink trail | World E2 | pulling the weight changes Mass and the trail bounces (not re-viewed this round; D2 is now close) | win-spring.jpg (earlier build) |
| Wiggle | D3 jittery thread, pink bead, ghost threads | E3 dot in dust with ellipse | Phenomenon D3 | liveliest and still fine in the row; E3 closes the gap after its fix | win-wiggle.jpg (earlier build) |
| Time remap | D5 draped film strip + ball + viewfinder | E4 loom of threads | Phenomenon D5 | film frames say "time" with Words Hidden; loom does not (E4 not re-viewed) | win-timeremap.jpg |
| Loop | D4 small film-frame racetrack | E5 same, large, with loop strip and Pulse | World E5 | fills the box, Pulse shows count/start/end; D4 is small | win-loop.jpg |
| Camera shake / Camera | D6 shake brackets over mountains | E6 lens cone | different concepts: D6 owns shake, E6 owns camera | E6 now fully responds (drag and Pulse) | win-camerashake.jpg, win-camera.jpg |
| Mask feather | B6 pebble, rim rub | D5 dotted window with fading rim contours | World D5 | rim contours show softness at rest (not re-viewed); B6 is better but still a blob | win-maskfeather.jpg (earlier build) |
| Stroke / Trim | C5 pen path ribbon | D4 knotted line being drawn | World D4 | trim is visible as a line being drawn under Pulse; C5 is a pen path = editor precedent. Flag: ends hidden at rest | win-stroke.jpg |
| Opacity | C6 cat behind misted pane, fingerprint wipe | D2 cat behind two frosted panes + blend | Phenomenon C6 | original wipe interaction; D2 is a derivative | win-opacity.jpg (earlier build) |
| Glass | B2 refraction ray | D1 slab + dispersion fan | World D1 | dispersion, thickness and Pulse, readout now slot-coloured | win-glass.jpg |
| Repeat | A3 diagonal file of tiles | B3 hex lattice with ghost slots | World B3 | ghost row/column show the repeat | win-repeat.jpg (earlier build) |
| Fill / Colour | (none) | D3 pigment strata (B2 colour gradient is a bead wire) | World D3 by default | calm now; the only candidate | win-fill.jpg |

Win count: Phenomenon 9 (Scatter, Stagger, Echo, Warp, Blur, Shadow, Wiggle, Time remap, Opacity), World 11 (Falloff, Glow, Noise, Easing, Spring, Loop, Mask feather, Stroke, Glass, Repeat, Fill) plus Camera/Shake split.

Winner screenshots are in `/Users/member_ottoto/rust_ae/design-sense-lab/research/shots/win-<concept>.jpg`. Those marked "earlier build" were copied from the previous round's pheno-*/world-* shots because that version was not re-viewed this round; the others are from this round.

## 4. The 10 most convincing toys overall
1. Phenomenon E4 Echo: a comet on a figure-eight with fading beads
2. Phenomenon B3 Shadow and sun: block, dotted floor, sun on a tether
3. Phenomenon C6 Opacity: cat behind a misted pane you wipe
4. World C5 Fractal Noise: calm contour map
5. World D1 Glass: slab, bending beam, dispersion fan, now with Pulse
6. World B3 Repeat: hex lattice with ghost slots
7. World C2 Glow: star over a threshold with dots lighting above it
8. World E1 Easing: footprints along a ribbon
9. Phenomenon E2 Variations: big face and a bank of eight, rewires from/to on drag
10. Phenomenon E5 Spin: striped top on a grainy floor (alternate: World E6 Camera or Phenomenon B5 bokeh)
