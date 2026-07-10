# Exit demo (M1 hero)

The M1 "exit demo": a **2-layer composite** — a real video background + a rectangle
sliding right with **easing (cubic-bezier ease-in-out)** — rendered to an mp4 from the CLI.
No UI involved; this is the headless pipeline (decode → canonical-space overlay → composite → encode).

`project.json` is a versioned project file: `input` (background video), `output` (mp4),
and an `overlay` whose `center.x` is keyframed from `-0.35` to `+0.35` over 2 seconds with a
`Bezier` ease. The rectangle is opaque red, `0.3 x 0.4` in canonical coordinates
(origin at the composition center, Y-up, height = 1.0), so it stays resolution-independent.

## Run it

Provide a background video as `input.mp4` in this folder. If you don't have one, generate a
test clip with ffmpeg:

```sh
# 2s, 1280x720, 30fps test pattern
ffmpeg -f lavfi -i testsrc2=size=1280x720:rate=30 -t 2 -pix_fmt yuv420p input.mp4
```

Then render:

```sh
cargo run -p motolii-cli -- export-project --project samples/exit-demo/project.json
```

The result `exit-demo.mp4` is the background clip with the eased rectangle composited on top.

> The automated end-to-end golden for this demo lives in `crates/motolii-cli/tests/exit_demo.rs`
> (generates a background clip, exports through the same path, then decodes the output and
> verifies the background shows through and the rectangle lands at the eased positions at
> start / mid / end). That test is the M1 exit-demo completion criterion.
