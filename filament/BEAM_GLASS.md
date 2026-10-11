# Glass block with sharp light (beam_glass)

A plain glass block that is dropped on a floor and throws exact, unblurred light: a dark sharp shadow, the
bright window of light that passed straight through, and thin spectral lines that start at the foot of the edges.

## Why the block has a chamfer

A mathematically perfect box never splits colour: parallel faces give the light back its entry direction, and at
n > 1.41 light that entered one face is totally reflected by the adjacent one. Real cut glass always has a
45 degree protective chamfer on every edge (0.1 to 0.5 mm on optical cubes, about 1.2% of the edge). That chamfer
is where the colour comes from. `CHAMFER` is its flat width as a fraction of the edge; up to about 2% it is not
visible as a change of shape.

## Files

| File | What it is |
|---|---|
| `beam_glass.h` | The light. Pure C++, no renderer. Polygon beams through a convex flat-faced solid (Heckbert and Hanrahan 1984), landed on the floor as flat polygons, brightness = flux / landed area (Sikachev et al. 2008). `trace` is the slow exact reference, `trace_fast` is what a frame uses. |
| `beam_glass_test.cpp`, `beam_test.sh` | Desk check: energy balance, agreement with rays walked one by one, fast against reference, top-view pictures `out/beam-top-*.png`. |
| `beam_light.mat` | Draws the landed polygons (additive, no blur) with the picture's own camera into a float buffer. |
| `floor_beam.mat` | A lit floor that adds that buffer, read by screen position. |
| `glass_solid.mat` | The block itself: each pixel follows its sight line through the same solid and looks at the scene without the block (`backdrop`) or at the room. |
| `beam_soften.mat` | One full-screen pass that softens the light and the block's shadow together, more the further the light flew (a sun with some width). Mip levels, fixed taps, no noise. |
| `ball_light.mat` | The floor light of every ball from one mesh, copied per ball and colour step on the GPU (see "Balls" below). |
| `cone_walk.mat`, `cone_light.mat` | The floor light of every cone and rod, on the GPU in two steps (see "Cones" below). |
| `flat_walk.mat`, `flat_light.mat` | The floor light of every flat-faced solid, on the GPU in two steps (see "Nothing per frame on the CPU" below). |
| `outline.mat` | The shadow outline of every solid, from a mesh built once per shape. |
| `shadow_only.mat` | Only with `FILSHADOW=1` (debug): a twin that draws nothing and casts Filament's own shadow. By default the shadow is the block's outline drawn as missing light, so it softens exactly like the light. |
| `glass_beam.mm` | The picture and the paced timing test. |

## Build and run

```sh
./beam_test.sh                                   # desk check, no renderer
for m in beam_light ball_light cone_walk cone_light flat_walk flat_light outline beam_soften floor_beam glass_solid shadow_only; do third_party/filament/bin/matc -a metal -p desktop -o $m.filamat $m.mat; done
./build_one.sh glass_beam
./out/glass_beam name                            # out/glass-beam-name.png (defaults: CHAMFER 0.03, DISP 10, SUNR 0.5, IBL 900, APERTURE 4.0)
DISP=20 IBL=600 APERTURE=4.3 ./out/glass_beam name   # stronger colour, a little more contrast
PACE=1 ./out/glass_beam pace                     # block and sun moving every frame, counts dropped frames
```

The materials also compile with `-a vulkan`, `-a opengl` and `-a all -p mobile`.

## What is exact and what is a lie

- Exact: where every patch of light lands and how bright it is (desk check: energy balance to 1e-15, picture
  within 0.5% of rays walked one by one). Branches carrying under 2% of the sun and light still inside after
  4 internal reflections are dropped (about 3% of the light).
- Deliberate lies: `DISP` spreads the colours further than real glass (default 10; 1 is real BK7). `CHAMFER`
  defaults to 3% of the edge (real optical cubes: about 1.2%). `SUNR` is half the sun's width in degrees
  (default 0.5; the real sun is 0.27), which sets how fast shadow and light soften with distance.
- The softening is an approximation in screen space (it ignores that the floor is seen at a slant).
- The grey of the shadow is plain fill light (`IBL`); light bouncing off the floor back into the glass is not traced.
- Not modelled: only the floor y = 0 receives light, only one block.

## Frame cost

- CPU per changed pose: `trace_fast`, 1.5 to 3 ms on an M4 at the defaults (three full traces plus exact follows of
  the patches whose colour actually moves by more than a pixel; more colour steps at higher `DISP`). Nothing is
  read back from the GPU.
- GPU: views for the light, the outline, the softening pass and the backdrop, plus two mip chains; about 6.5 ms
  at 1920x1080 on an M4. Paced at 60 Hz with block and sun moving every frame: 0 dropped of 600; at 120 Hz 119.4 fps.
- Before trusting a frame-rate number, check that nothing else holds the GPU
  (`ioreg -r -d 1 -w 0 -c IOAccelerator | grep "Device Utilization"`) and run `PACE=1 ./out/xcube_drop` as a control.

## Many solids, many shapes (glass_scene)

`glass_scene.mm` puts any number of solids on one floor (up to 64 in the face table): boxes of any proportion,
prisms, pyramids, an octahedron, anything convex with flat faces (`beam::from_planes`, `chamfered`, `placed`).

- The light is `beam::trace_cheap`: plausible, not exact. Each lit face is cut into `PIECES` x `PIECES` parts; one
  ray from the middle of each part decides where the whole part goes (into the glass, mirrored inside, out, on
  through another solid, down to the floor). Nothing is clipped or split, so the cost is parts x colour steps,
  whatever the number of solids. The exact trace (`EXACT=1`, `beam::trace_fast` on a `Scene`) needed 30 to 190 ms
  for two or three solids close together; the cheap one needs 0.3 to 0.5 ms for the same scenes and is a few
  percent of the picture away from it.
- Solids act on each other: light leaving one goes on through the next, a solid in another's sun takes that
  light first, and glass is seen through glass (`glass_solid.mat` follows the sight line once through up to two
  more solids, from a table of all faces that is refilled every frame; colours are told apart only at the last face).
- Any turn and any height: `TUMBLE=1` throws the solids about. `FRAMES=240` writes a run of frames to `out/seq`.
- A face that catches the sun flashes (`GLINT`, `GLINTDEG`).

Measured on an M4, 1920x1080, every solid and the sun moving every frame, paced at 60 Hz: 9, 40 and 64 solids
all delivered 60 fps with 0 dropped of 600; the light took 1.0, 3.7 and 5.7 ms of CPU. 9 solids at 120 Hz: 119.6 fps.
While tumbling, consecutive frames differ by 3.0 / 255 on average with the cheap trace against 2.7 with the exact
one (the cheap one adds about a tenth of flicker of its own).

Rods (`beam::rod`): the body is the same material told the kind (analytic). Their light is the sunlit side as one
mesh, every corner followed by one ray, the triangles landed where their corners land and as bright as they are
squeezed (brightness shared per corner, so no facets), at most `roundSteps` (8) colour steps read off the curve
through red, middle and blue. About 0.3 ms of CPU each.

### Balls: one shape, copied

A ball is the same from every side and every ball is the same ball at another size, so its light is one shape:

- `beam::ball_disc`: the sunlit side as 14 rings x 56 sectors, kept once on the GPU.
- `beam::ball_row` / `ball_table`: what a ray does inside a ball of radius 1 depends on its ring alone. 8 colour
  steps x 15 rings, worked out once (it depends on the glass only) and kept as a tiny texture.
- `ball_light.mat`: one instance = (ball, colour step). Each corner reads its row, is turned about the sun's axis,
  scaled by the ball's radius and dropped on the floor; its brightness is the sunbeam its triangles catch over the
  floor they land on. Per ball the CPU hands over four numbers (middle, radius) and nothing else.
- `beam::ball_light` is the same thing on the CPU, for the desk check and for `BALLCPU=1`; the GPU picture matches
  it to 1 / 255.

Left out on purpose (pretty and steady beats exact here): a ball's light is the same copy whatever stands in its
sun or in its light, overlapping lights and shadows simply add up on the floor; the faint light mirrored off the
outside of a ball is not drawn; light that leaves nearly flat or has flown more than 8 radii is faded to nothing.
Following every corner against the solids near it was built and thrown out again: 30 balls looked the same to
0.1 / 255, and its thresholds are exactly what makes a moving picture jump. The drawn light adds up to about three
quarters of what rays walked one by one put on the floor (the rest is the cap at the focus and the faint wide light).

Measured on an M4, 1920x1080, everything moving every frame, paced at 60 Hz, control (`xcube_drop`) at 60.0 fps:
30 balls 0.14 ms of CPU, 64 balls 0.18 ms, 64 balls tumbling 0.18 ms, all 60.0 fps with 0 dropped of 600. For
comparison 64 cubes take 6.9 ms (59.8 fps, 1 dropped of 600), and 30 balls took 9.7 ms before. At 120 Hz 64 balls
deliver 80 fps: the GPU (about 12 ms a frame for the whole picture) is the limit there, not the CPU.
240 frames of 30 balls tumbling (`out/glass-scene-balls-tumble.mp4`): the change from frame to frame stays between
1.7 and 1.9 / 255 with no step larger than 0.1 / 255 against its neighbours.

### Cones

`beam::cone(radius, height)`; the body is `kind` 3 of `glass_solid.mat` (closed form, as ball and rod). The light
is again one mesh on the GPU and nothing per cone on the CPU but three texels (`beam::cone_frame`):

- The mesh is the sunlit part of the surface: the side between the two lines where the sun grazes it (found
  exactly, so the mesh slides as the cone or the sun turns), and the base when that faces the sun.
- Light in a cone goes several ways, so each colour step is drawn as two sheets: light that leaves at the first
  face it meets inside, and light that is mirrored there and leaves at the second. How much takes each way is plain
  Fresnel, so a sheet goes dark by itself where the glass lets nothing out. (A third sheet, two mirrors, changed
  pictures of six cones by 0.01 / 255 and was dropped.)
- A sheet tears where its light crosses the rim inside (far side of the cone on one hand, base on the other). Light
  entering along one line from tip to rim all bends the same way, so where that happens is one quadratic
  (`beam::cone_split`), and the side is two patches that meet exactly on the tear. Tears after a mirror are not
  found exactly: there corners fade out, and a corner is only as bright as the dimmest of itself and its neighbours.
- Light that leaves upwards, too flat or too far fades to nothing first, and every corner keeps a place on the
  floor that moves smoothly whatever its light does, so no triangle is ever dropped.
- 8 colour steps; each corner tells the softening pass to blur by half the gap to where the next colour lands, so
  the steps run into one another (16 steps looked the same).
- Two passes: `cone_walk.mat` works every corner out once into a float texture (two texels per corner, one line per
  cone x colour step x sheet); `cone_light.mat` draws the mesh from it and reads the six neighbours it needs for
  the brightness. Doing the work in the vertex shader for every neighbour was six times too slow.
- `beam::cone_light` is the same thing on the CPU (`CONECPU=1`, and the desk check); the GPU picture matches it to
  1 / 255. Against rays walked one by one it puts 37 to 88% of the light on the floor, depending on the pose
  (missing: what is mirrored more than once, and partial mirrors).

What a cone throws: standing, almost all of its light lands under its own base (true to the rays; seen through the
glass, not on the open floor); tilted or lying it throws rainbow arcs and streaks. The inside of a tumbling cone
flickers like a kaleidoscope as faces start and stop mirroring; that is the body (`glass_solid.mat`), not the floor
light, and it is how a real cone behaves, but it is the least steady thing in the picture.

Measured on an M4, 1920x1080, everything tumbling, paced at 60 Hz, with two other jobs building and rendering on the
same machine (GPU utilisation 89%), control at 60.0 fps: 8, 30 and 64 cones 60.0 fps with 0 dropped of 600 (64 cones
three times in a row); the light takes 0.1 to 0.2 ms of CPU. A mix of 42 solids of all 14 kinds tumbling: 0 dropped of
600, three times. 240 frames of 12 cones tumbling with the bodies not drawn: the floor changes by 0.7 / 255 per frame
with no step above 0.05 against its neighbours; the largest local changes are shadow edges moving
(`out/glass-scene-cones-tumble.mp4`).

With eight busy loops hogging the CPU on purpose (control still 60.0): 64 balls 0 dropped of 600, 64 cones 1,
the mix of 42 solids 2, 64 cubes 1 dropped and only 58.6 fps delivered (their light, still worked out on the CPU,
took 11.6 ms on average and up to 52 ms). So what the CPU still does per frame (flat-faced solids and rods) is what
gives way on a busy machine; ball and cone do not. The helper threads are now started once and wait
(`detail::Workers`): starting ten threads anew every frame stalled for up to 130 ms when the machine was busy.

### Nothing per frame on the CPU

The rule (the user's, 2026-10-11): the CPU hands over where each solid stands and nothing else; whatever depends
on the pose or the sun is worked out on the GPU from data built once per shape. Reason, measured: with eight busy
loops hogging the CPU, 64 balls (GPU) dropped no frame while 64 cubes (their light then traced on the CPU, 0.1 ms
each) fell to 58.6 fps with stalls of up to 52 ms.

- Flat-faced solids: `beam::flat_shape` cuts every face into pieces once, in the shape's own space, and builds a
  table and a mesh. Per frame `flat_walk.mat` does what `beam::route` did (one ray from the middle of each piece:
  in, mirrored inside while the glass cannot let it out, out, on through other solids, down) and writes where
  three corners of the piece land; `flat_light.mat` draws the pieces from that (a piece goes where its middle
  ray goes by one affine map). Colour: 8 steps, each smeared half way to where its neighbours land, worked out
  exactly per pixel (the share of that stretch for which the piece, sliding along it, covers the pixel), so the
  steps cross-fade into a continuous spectrum with edges as sharp as before. (Blurring the steps into one another,
  as the cone does, made mush of the sharp bands; stretching with a uniform thinning left seams.)
  `FLATCPU=1` is the old CPU path, to compare: the pictures agree to 0.04 / 255 on average for one cube.
- Rods are drawn by the cone's two passes: a rod is a cone with its tip cut off so far down that both ends are as
  wide (`beam::Taper`; the mesh has a fourth patch for the top). The old per-point light of round solids is gone.
- Shadow outlines: `outline.mat` slides the sunlit part of a mesh of the shape along the sun (for a cone, between
  the two lines where the sun grazes it, found in the shader). `OUTLINECPU=1` is the old path; same picture to 1 / 255.
- Helper threads are started once and wait (`detail::Workers`); creating ten threads every frame stalled for up
  to 130 ms on a busy machine.

What the CPU still does per frame, for 64 solids 0.13 ms in all (it was 3 to 7 ms): place the solids, and copy
their places into the pose table, the table of world-space faces for glass seen through glass (`sceneTex`; this
one is still built from the placed solids and should become a small GPU pass), and the ball and cone tables.

Open after this move:
- GPU time: see "GPU time, what was cut" below. Still open there: a frame has little room when another program
  takes the GPU, and four matte blocks with 36 tumbling solids do not hold 60 Hz.
- Flat-faced solids flicker when they tumble: patches of light, often strongly coloured, change from one frame
  to the next (up to 50 / 255 in a 24 pixel block). Measured later the same day: this is mostly the light itself,
  not the piece method. Nine cubes, the same 240 frames at an eighth of the speed (0.3 degrees a frame), area that
  changes by more than 10 / 255 in one frame as a solid patch: 1384 px a frame (median) with the exact trace;
  what the piece method does differently from the exact trace changes by 28 px a frame (worst frame 2924).
  At full speed (about 2.4 degrees a frame) the exact picture has as many one-frame jumps as the cheap one.
  What moves the number: the speed of the turn (half the speed: a third of the strongly changing area) and the
  sun's width (`SUNR=1.5`: half); `DISP=4` hardly at all.
  The piece method's own pops do exist (`flat_pop_test.cpp`, one cube turned in 0.5 degree steps against the exact
  trace: 2 or 3 a turn at `PIECES=4`, none from 6 up), but `PIECES=6` costs 5 ms of GPU for 64 cubes.
  Tried and thrown away: dimming a piece that is about to change its way (so that it changes while dark). No pops
  in the desk check, but light that leaves through a chamfer is always "about to change" (a chamfer is thinner
  than a piece), so the coloured light went with it; a tenth of all the light was lost.

### Dents and holes (star, washer, letters)

`Solid::EXTRUDE` (beam_extrude.h): an outline of straight lines and arcs with any number of holes, pushed through a
thickness; `beam::star`, `washer`, `letter('A' 'B' 'M' 'O' 'R')`, written by hand as geometry. Body: `kind` 4 of
`glass_solid.mat` (a 2D ray against the outline's edges plus the two flat faces; may leave into a dent and enter
again). Light and shadow, all on the GPU from tables built once: `extrude_find.mat` finds, for every stretch of
wall, where along it the light changes its way (up to two places, by halving), `extrude_walk.mat` works out what
becomes of the light at five points per stretch, `extrude_strip.mat` and `extrude_cap.mat` draw strips along the
walls and the outline's own triangles from that. `ONLY=holes`, `EXTRUDECPU=1` for the CPU twin.
Open: no round ring (torus); not in `beam_test.sh` (`extrude_check.cpp` compares against rays but has no pass or
fail); a third change of way on one stretch of wall is smeared and can pop; light within about 6 degrees of total
reflection is dimmed away on purpose; light that leaves the solid and meets it again across a dent flies on as if
nothing were there; the header of `beam_extrude.h` needs a long explanation, which says it is too complicated.

### Walls and matte blocks (the stage)

`WALLS=1|2`, `PROPS=1..4` (or `PROPAT=...`), `STAGE=1`. The room (floor, walls) and the blocks are two small tables
built once (`beam::make_stage`). Every material that puts light or an outline on the floor ends with the same two
calls: `recvDrop` (recv_drop.glsl, vertex) takes the corner back from where it lands on the floor ALONG THE SUN to
where it started and drops it on a receiver instead; `recvFinish` (recv_finish.glsl, pixel) keeps a pixel only where
that receiver really is, and not where a block stands in the light's way. Two ways of choosing the receiver: a
material with large triangles (the outlines, the caps of solids with holes) is drawn once per receiver (the floor,
each wall, three faces per block), exact; a material whose light is a fine mesh (ball, cone, rod, pieces of flat
faces, strips) lets every corner choose (`recvChoose`) and is drawn once for the room and once more for the blocks.
What a block hides from the camera is left to the depth buffer (the blocks are drawn into both pictures first).
The surfaces are `recv_beam.mat`; the softening pass keeps light on a block apart from the floor seen behind it
(`marks`). The glass bodies find walls and blocks along their sight lines (`glass_solid.mat`, `recvFirst`).
The lie: light that the glass has bent lands on a wall or block as if the floor's picture had been thrown on it by
a projector standing in the sun (wrong by height above the floor x how far it was bent); shadows and
straight-through light are exact; light that never comes down to the floor is on no wall. (The worker who built
the stage had made ball and cone land along their true ways instead; that was not taken over: it collided with
the cone's rework and was the part its author called complicated.)
Limits: the camera must be inside the room; walls and blocks stand still during a run; at most 2 walls and 4
blocks; glass cannot stand on a block in this program. Without any of the switches the picture is unchanged (1 / 255).

### A bar pushed along a path and turned as it goes (knots, twisted rings)

`SWEEPS=n` (glass_scene) floats n such bars over the rest: a knot of three-sided bar, a ring of four-sided bar that
rises and falls, a knot of six-sided bar (`beam_sweep.h`: `sweep_closed(path, sides, radius, turns)`; any closed
path, any regular profile up to eight sides; `SWEEPTURNS`, `SWEEPY`, `SWEEPSIZE`).
The one idea: at any place along the path the bar is taken to be a STRAIGHT bar with the profile as it stands
there. Everything is worked out against that straight bar's few sides.
- Body, `sweep_glass.mat`: the sight line is bent in, mirrored inside while the glass cannot let it out, bent out
  once per colour, and looks at the backdrop like every other solid.
- Light and shadow, `sweep_light.mat`: the bar's own surface drawn triangle by triangle; every corner follows the
  three corners of its triangle through the bar (one ray each, per colour step, 12 steps) and the triangle is
  drawn where they land, as bright as the sunbeam it catches is to the floor it lands on. A triangle whose corners
  went different ways is put on one spot and not drawn. It lands on walls and blocks through recv_drop.glsl like
  the rest. The shadow is the same mesh slid along the sun, as missing light.
- Per frame the CPU hands over one pose per bar.
Left out on purpose: that the path bends and the profile turns while the light is inside (fine while the path
bends gently against the profile's width); light or sight lines that leave the bar and meet it again; other
solids in its light, and it in theirs; glass seen through it and it seen through other glass (each sees the
backdrop); it does not block other solids' sun. Brightness is per triangle (the mesh is fine: 360 places along).
Measured: three bars cost about 4 ms a frame on top of 40 tumbling solids (16.6 against 12.3 ms); paced at 60 Hz
with another program holding the GPU: 2 dropped of 600; the bars alone 0. (Before the triangles that went
different ways were left out: 74 ms.) `out/glass-scene-sweep-mix.png`, `.mp4`.

### GPU time, what was cut (2026-10-11, evening)

`BENCH=n` (glass_scene) draws n frames one after another and waits for each: what one frame costs, without the
waiting for a slot that hides it in `PACE`. Read the least of the times: with little to do the GPU slows its clock
and the middle one goes up. Three things were cut, 64 tumbling cubes went from 17.9 to about 14 ms a frame:
- Filament's shadows were still switched on in both pictures though nothing casts one (every shadow here is an
  outline drawn as missing light): the floor worked out a soft shadow at every pixel, twice, from an empty map.
  Off: 2.5 ms. The picture is the same (3 pixels of 2 million differ by more than 2 / 255).
- Glass seen through glass asked all 64 solids at every pixel (3 ms; the loop itself, not what it finds). Now each
  solid marks, once per corner, the solids that can be seen through it: those behind it as the camera sees it,
  inside a cone (`NEARCONE`, 0.8; 0 asks all as before). Left out on purpose: a solid in front or far to the side,
  seen by a sight line the glass has thrown right round. About 2 % of the pixels of a picture of 64 cubes differ;
  it is no less steady in motion (area changing in one frame: 11946 against 12253 px).
- Matte blocks made the light of round solids cost 10 to 20 ms more (36 of them, two blocks). Two causes in
  recv_drop.glsl: the copy of the light for the blocks was rasterised whole, as large as it lies on the floor, and
  cut away pixel by pixel (now a corner with no business on a block goes to the middle of a block, and its
  triangles are not drawn); and every pixel near a block asked every block whether it was in the way (now asked
  once per corner, recvClear, for the materials whose light is a fine mesh). Two blocks now cost 0.5 to 2.5 ms.
  The edge of a block's shadow in light made of flat pieces is softer than before (a piece is 0.25 across).
Paced at 60 Hz with another program holding the GPU (idle reading 76 to 95 %; the control, xcube_drop, 0 dropped):
64 cubes 0 dropped of 600, 64 of all kinds 0, 30 with holes 0, 64 rods 3, 64 cones 2; two walls, two blocks and 36
tumbling: 38; with four blocks: 126 (47 fps). Before the cuts, on a quiet machine, the same list read 9 to 19, 59
to 63, 0, 26, 0, 158 (44 fps).

### The budget: 6 ms a frame, and how it is measured (2026-10-11, night)

The user's rule: the GPU is always shared (other effects will take it), so "fits in 16.7 ms" is not the test; the
glass may use 6 ms of GPU a frame at 1920x1080. Measured by how many frames a second come out with no waiting at
all, `PACE=1 PACE_HZ=2000 PACE_FRAMES=2500`: the GPU is then kept busy and at full clock (with little to do it slows
its clock and `BENCH` and the paced "GPU time" both read too high), and 1000 / (frames a second) is what a frame
costs. 6 ms = 167 frames a second. Switches that leave single stages out, for timing only (the picture is wrong):
`XLIGHT XSHADE XMIPS XSOFT` (and in `BENCH`: `XALLLIGHT XBACKDROP`); `SS=0.5` draws light and outline at half size
each way, `BACK=0.5` the backdrop.
64 tumbling cubes: 14.9 ms. The glass bodies 6.5 (of it glass seen through glass 3, nine looks at the backdrop
instead of three 1.3), the floor light 3.7, the rest 4.8 (2.7 with nothing in the scene).
The floor light without the bodies, 8.5 ms: drawing the light triangles 3.8, the mip chains 1.2, the softening
pass 1.0, the outlines 0.7, everything else 3.4. Light and outline at a quarter of the size each way: 6.5, so most
of the triangles' cost is their number (nine copies of every piece, one per colour step), not the pixels.
The backdrop at a quarter of the size or with half the bytes: 0.6 ms at most. Reading pictures is not what costs
in the bodies; working out is.

### Experiments towards 6 ms (2026-10-11, night): switches that are off by default

Decided by the user: the glass body stays our own (Filament's refraction loses the real shape, the inner mirrors and
the face-dependent rainbow); what merely repeats Filament is to go; the goal is all kinds together at 1080p in 6 ms.
All costs below by `PACE=1 PACE_HZ=2000 PACE_FRAMES=2500` (1000 / frames a second); the machine was busier than
earlier in the day, so compare within one table only.

What the "rest" of a frame really is (all kinds 64, nothing drawn, 9.6 ms): the table passes that work out where
light goes are 6 of it, and they run whether or not the light is drawn. `XWALK=1|f|e|w|c` leaves them out one by
one: solids with dents and holes 3.8 (find) + 3.1 (walk), cones and rods 1.8, flat faces 0.9 (they overlap: the sum
is more than 6). What is left, 3.3 ms, is the pipeline: softening 1.0, bloom about 1.0, backdrop 0.6, mip chains
0.4, FXAA 0.2, the rest 1.8 with every stage off. The CPU is not in it: main thread 0.17 ms, backend 0.36 ms a frame.

`SSRBACK=1` (glass_solid_ssr.mat): the bodies look at the copy of the opaque scene that Filament makes for its own
screen-space refraction (`sampler0_ssr`; an unlit material may read it once it declares `refractionMode :
screenspace`), and the backdrop view is not rendered. It works, and it buys nothing: 14.3-15.0 against 15.0-15.2 ms
for 64 cubes, 17.3 against 17.0 for all kinds (Filament's copy with its blurred mip chain costs what our view cost);
4 % of the pixels differ. Not adopted.

`RAINBOW1=k` (flat_walk1.mat, flat_light1.mat): the spectrum in k stretches instead of 8 steps; a piece is drawn once
per stretch between where the stretch's two ends land, and every pixel takes the colours of the part of the stretch
that covers it (difference of a running sum of the spectrum, `rainbow`). Colours are continuous and the light adds
up to the same (208.44 against 208.47). 64 cubes without bodies: 8 steps 8.7-9.2 ms; k = 1: 5.5, 2: 6.9, 3: 7.5,
4: 8.5. But it is less steady, at every k. Nine cubes at 0.3 degrees a frame, area where (picture - exact picture)
changes by more than 10 / 255 in one frame, median / worst: 8 steps 29 / 2927 px; k = 2: 2862 / 26091; 4: 1719 /
22686; 7: 702 / 14758. With k = 1 the large one-coloured windows (red and blue leaving through different faces)
are gone; k = 4 has them back, at no saving. Whole bands change colour from one frame to the next: the ends of a
stretch are forced along the way its middle takes, and fall back to the middle's direction when they cannot follow.
Not adopted as built.

`POLYTEST=n` (floor_poly.mat): every floor pixel works out n soft-edged four-sided patches from a float table, no
picture in between. With the floor filling the screen: 8 patches +0.4 ms, 16 +0.9, 32 +1.8, 64 +3.7: about 0.06 ms
for one patch laid over the whole screen. One cube's light today lies 7.7 patches deep on a lit floor pixel on
average (median 4, 95 % 29), colour steps counted separately. So soft-edged pieces evaluated where they land are
affordable; what they would remove is the light and outline pictures, both mip chains and the softening pass.

`STDGLASS=1` (std_glass.mat): the bodies by Filament's own refraction, to compare (0.8 ms against our 6.6; the glass
then reads as a faint film). See also the audit notes in the project memory.

### A body drawn from regions instead of pixel by pixel: first trial, a regular net (`VBODY=1`)

The user's point: the dear work must not grow with the number of pixels; move it from pixels to shapes and regions.
Where it grows today (64 tumbling cubes; `RES=2` / `RES=4` render 3840x2160 / 7680x4320):
whole frame 15.8 / 47.6 / 169.5 ms at 1080p / 4K / 8K; the glass bodies alone 7.2 / 26.2 / 97.0 (every pixel follows
its own sight line); drawing the floor light 3.6 / 9.4 / 29.7; the rest 4.9 / 12.1 / 42.7.

The trial (glass_nodes.mat, glass_vert.mat; flat-faced solids, their four-sided faces; not on a stage):
- a table pass works out, at 17 x 17 points on every such face, where the sight line through the glass ends up
  (through other solids too: the search is done per point, not per pixel), for the red and the blue end;
- the face is drawn as a net: 4 x 4 cells, a cell replaced by its four quarters (8 x 8) where it fails, a quarter
  drawn the old way (per pixel) where that fails too. A cell passes if its corners, the middles of its sides and its
  own middle all went the same way and the middles look within `VTOL` pixels of what the corners would give (the
  middles ARE points of the table: nothing extra is worked out);
- a pixel of a passed cell only reads the backdrop: once, or 5 / 9 times along the line from red to blue where the
  two ends are more than 1 / 6 pixels apart.
`VSHOW=1` paints the cells (green large, blue small, red per pixel).

Measured, 64 tumbling cubes:
| | 1080p | 4K | 8K |
|---|---|---|---|
| per pixel (today) | 14.1 ms | 41.5 | 149.3 |
| net | 16.3 ms | 39.4 | 128.2 |
| the same two without glass seen through glass | 11.6 / 11.5 | 31.7 / 28.9 | 112.4 / 96.2 |
- Share of the glass pixels: large cells 30 %, small cells 16 %, per pixel 54 % (42 % without glass through
  glass), the same at all three sizes and nearly the same for a tolerance of 0.5, 1, 2 or 4 pixels: cells fail
  because a boundary between two ways crosses them, not because of interpolation error.
- Picture against the per-pixel one: mean 1.9 / 1.4 / 1.4 of 255 at 1080p / 4K / 8K, 8.1 / 6.0 / 6.0 % of the
  pixels differ by more than 8 (part of it: the table asks all solids, the per-pixel way only the marked ones).
- Steadiness (0.3 degrees a frame, share of the picture changing in one frame as a solid patch by more than
  10 / 255, median): per pixel 0.576 / 0.505 / 0.402 %, net 0.577 / 0.494 / 0.413 %: the same.
- The table pass: 1.6 to 1.8 ms for 64 cubes. Running the pass with every texel returning at once: 0.1 ms. Without
  the search through other solids: 0.3 to 0.55 ms. So it is the search (1.2 ms), which grows with the square of
  the number of solids (8: 0.18, 16: 0.34, 32: 0.73, 64: 1.8 ms).
  Working out only one of a point's three texels costs the same as all three (9.8 ms both): the GPU shades 2 x 2
  texels together, and the three texels of a point, lying side by side, share one walk. Laid out apart (one block of
  the table per kind of number) one kind costs 0.9 ms and all three 2.4: worse. Kept side by side; a point packed
  into ONE texel would cost about 0.9 ms, half, not a third.
Verdict: no gain at 1080p, 5 % at 4K, 14 % at 8K. The cell map shows why: the boundaries are a few nearly straight
lines per face (the solid's own far edges seen through the face, where total reflection sets in, the outlines of
other solids), and a regular net pays for each line with a band one cell wide. A line is paid for as an area.
Next (not built): take the boundaries themselves as geometry and cut the drawn regions along them.

### Second trial: regions cut along the boundaries, in two layers (`VBODY=1` as it is now)

Prior art (research): the facet tree / beam tree for gemstones (Guy & Soler 2004; Voelker & Hamann 2012) splits a
convex polyhedron's picture into polygons of one face sequence each; both clip on the CPU or read back from the GPU,
which is not allowed here, and no code was found. Clip distances cannot be written from a Filament material (v1.77.2:
only for instanced stereo). No name or implementation was found for what is done below.
Before building, counted from the point table (`VNODEMODE=3 VDUMP=file`, ALONE): 64 cubes: 96.8 % of the points leave
after meeting at most 3 faces inside; a face shows 12 different ways (median; most 19); its 8 commonest cover 97.2 %.
Nine kinds of flat-faced solids: 84.3 % within 3 faces, 91.6 % within 4; 13 ways a face (most 41); 8 cover 93.2 %,
16 cover 98.3 %.

How it works now:
- the solid is drawn the OLD way as ever (glass_solid.mat), and a hair in front of it a CHEAP LAYER (glass_vert.mat,
  drawn first: `priority`). What the cheap layer covers, the depth test takes away from the old way before it is
  worked out; the old way fills whatever is left. So the picture is always whole.
- cheap layer, per four-sided face: 4 x 4 cells from the point table; failed cells as four quarters; and a quarter
  that fails too is drawn once for the way each of its corners and its middle went (the table now holds the way as
  the list of faces itself): every corner of the quarter is MADE to go that way (`forced`) and carries how far it is
  from not going it: `room` (distance to the edge of each face it is made to meet) and `turn` (how far past or short
  of total reflection). A pixel is kept where both are above nothing (`discard` otherwise). A boundary so costs a
  line, not a band of cells.
- `VSHOW=1`: green large cells, blue small cells, yellow cut along a boundary, red left to the old way.

Measured, 64 tumbling cubes, no glass seen through glass (control scene steady at 2.6 ms):
| | 480x270 | 1080p | 4K | 8K |
|---|---|---|---|---|
| per pixel | 5.0 ms | 11.2 | 31.5 | 112.4 |
| two layers | 6.5 ms | 12.0 | 28.1 | 91.7 |
| no bodies | 4.2 ms | 8.1 | 20.2 | 68.0 |
With glass seen through glass (search still per point in the table; ways through other solids are left to the old
way): per pixel 14.2 / 41.7 / 151.5, two layers 15.5 / 35.3 / 112.4 at 1080p / 4K / 8K.
- Glass pixels (nine cubes): large cells 38.5 %, small 20.2 %, cut along a boundary 20.4 %, left to the old way
  20.8 % (the regular net left 42 %). What is left: the chamfer faces (thin, always the old way) and the places
  where several boundaries meet.
- At 4K the body costs 7.9 ms against 11.3: cheap layer's pixels 3.1, old way's remainder 3.8 (a third of what the
  old way costs for every pixel: the depth test does its work), the rest fixed.
- Fixed, whatever the picture's size: point table 0.7 ms, the cheap layer's corners about 1.3 ms (was 1.8 before
  faces turned away were dropped early and cells went from six corners to four). This is why 1080p is still 0.8 ms
  WORSE than per pixel.
- Picture against per pixel (nine cubes): mean 0.55 of 255, 2.0 % of the pixels over 8. Seen: strongly coloured
  facets are paler. Where the red and the blue end look at different KINDS of thing (one the floor, one the sky) the
  cheap layer takes both from the middle.
- Steadiness (0.3 degrees a frame, share of the picture changing in one frame as a solid patch by more than
  10 / 255, median): per pixel 0.589 % / 0.504 % (1080p / 4K), two layers 0.609 % / 0.555 %; the difference between
  the two pictures itself changes on 0.06 % / 0.04 %.
Verdict so far: the dear work no longer grows in step with the pixels (body 3.1 -> 44 ms per pixel against 3.9 -> 24
from 1080p to 8K), but at 1080p nothing is gained yet, and it only covers flat-faced solids' four-sided faces.

### The two layers, corrected; and the light's pictures at a fixed size (`LIGHTH`)

Corrections to the two layers (same day):
- the point table's own places in the picture were a few pixels off (cells drawn from them differed from the old way
  by 3 of 255 on average even at a tolerance of 0.05 pixels; why is not found). Every drawn corner now works its way
  out afresh (`forced` with the way the table names); the table is one texel per point (way, a place for the test,
  weight) and glass_cells.mat works out once per cell whether it passes. Both table passes together: about 0.2 ms.
- colours: nine wavelengths as the old way, along the curve through where red, the middle (587.56 nm, 0.2104 of the
  way) and blue look, each placed by 1 / wavelength^2; a colour that leaves without coming down looks at the sky.
  Read once where red and blue are less than a pixel apart and both come down.
- the cell test's middle is compared with what two triangles draw (half way along the shared side).
- keeping a way a little past its boundary (`VSLOP`) closes cracks but fills them with the wrong way (old-way share
  21 -> 13 % at 0.02, error of the cut cells 3.5 -> 6.9): not used.
Now, 64 cubes, no glass through glass, picture against the old way: all glass pixels mean 1.8 of 255, 6.3 % over 8,
1.5 % over 25 (no dispersion: 0.2, 0.2 %, 0.0 %). Shares: large cells 39.7 %, small 18.3 %, cut along a boundary
20.6 %, old way 21.4 % (about half of that the chamfer faces, which are six-sided and 6 % of a cube's surface).
The cheap layer costs about 1 ms whatever the picture's size (614 thousand corners a frame for 64 cubes).

The light's pictures at a fixed size: `LIGHTH=n` makes the light, the outline and the softened picture n lines high
whatever the final picture's size; `LEAST` (1.2 by default then) spreads at least that many of their pixels, or the
pixel steps along a sharp edge show as stairs through the glass (they still show as a faint wave).
| 64 cubes, no glass through glass | 1080p | 4K | 8K |
|---|---|---|---|
| today (per pixel, light at the picture's size) | 12.1 ms | 30.6 | 109.9 |
| two layers | 11.6 | 28.2 | 93.5 |
| light fixed at 1080 lines | 11.2 | 23.5 | 68.0 |
| two layers + light fixed at 1080 lines | 11.7 | 20.0 | 51.8 |
| two layers + light fixed at 720 lines | 9.7 | 17.8 | 51.6 |
Picture at 4K against light at 4K: tumbling cubes 0.05 of 255, 0.01 % of pixels over 8 (720 lines: 0.10, 0.02 %);
23 solids standing on the floor 0.18, 0.17 % (with `LEAST` 1.2: 0.24, 0.30 %). Steadiness at 4K: the same (share of
pixels changing by more than 10 / 255 in a frame 0.557 % / 0.485 % / 0.471 % for light at 4K / 1080 / 720 lines).
Both switches are off by default.

### Where it stands (2026-10-11, M4, 1920x1080, everything moving, 60 Hz, quiet machine, control 60.0 fps)

0 dropped of 600: the default 23 solids; 42 of all kinds tumbling; 64 balls; 64 cones; 30 solids with holes; two
walls with 23 solids; two walls with 36 tumbling. Not holding: 64 cubes and 64 rods tumbling (26 dropped, 57 fps),
64 of all kinds tumbling (59 dropped, 54 fps), two walls and two blocks with 23 solids (3 dropped), with 36
tumbling (45 fps). The CPU's share is 0.06 to 0.18 ms a frame in all of these; what gives is GPU time.

## Sources

- Heckbert, Hanrahan, "Beam tracing polygonal objects", SIGGRAPH 1984.
- Sikachev, Tisevich, Ignatenko, "Interactive caustics from polyhedra", GraphiCon 2008.
- Wyman, Sloan, Shirley, "Simple analytic approximations to the CIE XYZ color matching functions", JCGT 2013.
- N8python/diamonds `totalInternalReflection` (MIT) for the bounce loop of `glass_solid.mat`; see `third_party/NOTICE-glassbox.md`.
- Glass data: Schott N-BK7, nd 1.5168, Abbe 64.17.
