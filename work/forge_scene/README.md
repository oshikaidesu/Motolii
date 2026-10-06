# RESULT: FAIL — reference only

This directory is the 2026-10-06 "30-minute AE" prototype. It is kept as artwork/reference. **Do not optimize it, port it, or build on its path.**

Why it fails (all of these, any one is enough):

1. No 60 fps interactive path: one seek costs 0.84–0.94 s against a 16.67 ms budget (about 50–56x).
2. It breaks "GPU readback during playback = 0" (screenshot readback, PNG/PFM round trips).
3. A filesystem intermediate per frame (PNG, PFM, temp files).
4. A subprocess per frame (ffmpeg, oidnDenoise, wgsl-fx, post.sh).
5. The video frame goes through the CPU and is uploaded again (ffmpeg pipe -> 8.3 MB -> texture).
6. Denoise is CPU and offline (OIDN CLI on 8-bit tone-mapped PNGs).
7. The Stage does not show the GPU output; it reads a PNG and decodes it in Flutter.
8. Motolii wrote its own light transport (bounce loop, throughput, termination, medium state, environment, emission).
9. Motolii wrote its own scheduling and evaluation (driver state machine, shell pipelines, animation formulas in rerun-world, effect contract in wgsl-fx).
10. "Play" is not playback: a seek loop that shows about 1.1 images per second, 3 timeline frames each, no audio.

What it is good for: the picture (thick blue glass, refracted live-action video, SVG, reflection) as a look reference, and the measured list of where responsibility leaked.

The constraint it exposed (architectural, not an optimization target):

> Interactive path is a continuous GPU path. No readback, no files, no per-frame process boundary. 16.67 ms is an architectural constraint, not an optimization target.

Measured on M4 at 960x540 (see friction-log.md and the audit): ray-query glass costs about 17 ms per sample, so even 1 spp of this path tracer spends the whole frame budget before any denoise.
