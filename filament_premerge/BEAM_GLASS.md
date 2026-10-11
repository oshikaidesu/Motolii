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
- GPU time. A frame of 64 solids takes 13 to 18 ms of GPU on an M4 (about 6.7 ms with nothing in the scene, about
  6.7 ms for the glass bodies of 64 cubes of which 4 ms is glass seen through glass, the light 3 ms for 64 balls
  or cubes, 8 ms for 64 cones, 10 ms for 64 rods). It holds 60 Hz on a quiet machine for everything but 64 rods,
  but has no room: when other jobs take the GPU, frames drop, and so does the control once the GPU is saturated.
- Flat-faced solids pop when they tumble: small patches of light, often strongly coloured, appear and vanish from
  one frame to the next (up to 50 / 255 in a 24 pixel block). It is the piece method itself (a piece is sent
  whole wherever its middle ray goes, each colour step on its own), the same on the CPU path. The cure is to cut
  every piece exactly where its light starts to go another way; the three carried corners already give, at every
  face met, how far each corner is from the edges of that face, which is all a pixel needs to know.

Not covered yet: solids with dents or holes (text, stars, rings), light landing on anything but the floor
y = 0, a real physics engine driving the solids, more than one sun. Flat-faced solids and rods are still worked
out on the CPU per solid; the same "one shape, copied" would fit any solid whose light does not change with its
turn about the sun's axis, and the flat ones could at least reuse their light between frames in which neither they
nor the sun turn.

## Sources

- Heckbert, Hanrahan, "Beam tracing polygonal objects", SIGGRAPH 1984.
- Sikachev, Tisevich, Ignatenko, "Interactive caustics from polyhedra", GraphiCon 2008.
- Wyman, Sloan, Shirley, "Simple analytic approximations to the CIE XYZ color matching functions", JCGT 2013.
- N8python/diamonds `totalInternalReflection` (MIT) for the bounce loop of `glass_solid.mat`; see `third_party/NOTICE-glassbox.md`.
- Glass data: Schott N-BK7, nd 1.5168, Abbe 64.17.
