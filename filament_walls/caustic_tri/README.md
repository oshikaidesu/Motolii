# caustic_tri — spectral caustics, Filament only

Glass → coloured caustic on a floor, in real time, using **only Filament materials** (no compute, no wgpu, no extra
graphics API). Run it:

```sh
./caustic_tri/build.sh      # matc -> .filamat (Metal; Vulkan/OpenGL also compile, see below)
./link.sh caustic_tri       # builds caustic_tri.mm, runs the checks, writes out/caustic-tri-floor.png
```

## Passes (per glass object, all inside Filament)
1. `cap_front.mat` / `cap_back.mat` — unlit, data in the colour: (octahedral normal, distance along the sun). Nearest front face / nearest back face.
2. `hits.mat` — one fullscreen pass over a `7n x n` RGBA32F target. For every light-view texel and 6 wavelength bands: refract in, march + bisect to the back face, refract out, land on the floor. Cauchy `n(l)`, same 6 bands / XYZ weights as luma.gl.
3. `tri.mat` — every light-view cell is drawn as a triangle at its refracted corners. Brightness = `oldArea / newArea` (Wallace / Renou area ratio, so light is conserved). Additive RGBA16F, rgb = CIE XYZ. A second draw (`mode 1`) paints the unrefracted footprint (the glass's shadow).

Borrowed: refraction, ray-march idea, spectral constants (luma.gl, MIT); area-ratio triangles (Wallace 1987 / Renou 2023).

## Checks that run every time
* **sphere profile** — a sphere under a vertical sun. The cumulative energy inside radius r on the floor is compared with the exact answer from a 1-D refraction solved on the CPU. Last run: worst gap 0.0053 of the beam (`PASS`).
* **footprint area** — shadow area equals `pi R^2` within 1%.

## Things Filament did that cost an afternoon
* **Metal render targets are top-down**: texture row 0 is the top of the picture. Every pass here uses `v = 0 is the +up side`. A bottom-up assumption only fails off the centre row, so a sphere at the centre cannot reveal it.
* **One renderable draws at most 32767 instances.** The cells are split into chunks with an `instanceBase` parameter. Beyond it the draw silently drops the rest (half the floor was missing).
* There is no `getInstanceIndex()`; use `gl_InstanceIndex` (Metal/Vulkan) or `gl_InstanceID` (GL).
* Custom vertex data is read with `getCustom0()` (attribute `CUSTOM0`), not `material.custom0`.
* An RGBA32F target needs `BLIT_SRC` to be read back.
* The identity ortho camera at the origin keeps `material.worldPosition` free of Filament's camera-relative origin shift.

## Not verified yet
* Only run on Metal (Apple M4). `matc` compiles all four materials for Vulkan and OpenGL; running them there is untested.
* Timing is wall time of a `flushAndWait` frame (CPU encode + GPU), 2 objects, capture 256, map 1024: about 5 ms. The wgpu prototype measured 5.9 ms GPU for the same scene. Not a GPU-only number.
* The brightness limit (`ratioCap`, default 64) drops energy at fold caustics on purpose; the sphere check turns it off.
