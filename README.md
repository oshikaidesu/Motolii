# Motolii

**[日本語: なぜ、もう一つ映像制作ソフトを作るのか](MANIFESTO.ja.md)** — After Effects の重さ、AviUtl からの移行、ソフトごとのエフェクト再発明、そして「映像制作における VST」について。問題設定と長期方針の要約は [VISION.ja.md](VISION.ja.md)。

> **Everything you hand to After Effects becomes a flat rectangle.** A 3D scan, a particle field, a camera path, an audio waveform — the moment they enter the composition they are pressed into planar layers that no longer remember what they were.

Motolii is built on two *what ifs* that were both nearly real.

**What if a compositor never flattened meaning?** Motolii's stage is [Rerun](https://rerun.io) — a semantic data engine built for robotics and computer vision, where a point cloud stays a point cloud, a mesh stays a mesh, a path stays a path. Compositing on top of that store means you combine *meanings*, not rasters. The concrete promise, and it is kept today: **one effect, written once, lands on a video, a text outline, a mesh and a point cloud alike** — a turbulent displacement bends the silhouette of all of them, not a picture of them.

**What if AviUtl's culture had met After Effects' grammar?** For two decades a free, local Japanese editor was bent by its extension community into shapes its author never imagined, and an entire MV/MAD culture grew in that gap. That crossing never happened — the freedom grew on one island, while AE's depth stayed sealed behind a vendor SDK, and no bridge was built between them. Motolii is the bridge that was never built: the AE-family editing grammar your hands already know, with extension freedom as a constitution. Save a shader file and it is on the shelf; paste a Shadertoy and it runs; the host, not the plugin, owns time.

Both futures were plausible. Neither happened. Motolii is being built so they can — and the whole build is public: every design decision is a numbered ruling in [the decision index](docs/decision-index.md), every law a document you can read before you write a line.

## What works today (September 2026)

The effect system is the part that is ahead of everything else, and it is the reason this project exists.

| You write | It runs on | How |
|---|---|---|
| A **field** (`fn field(in, params) -> offset`) | video and image boards, text and shape outlines, meshes, point clouds | boards become a grid, outlines become a mesh with anchored strokes, meshes and clouds move their vertices — [field model](docs/vism-field-model.md) |
| A **pass** (an ISF fragment shader) | every material | boards are baked; meshes and clouds get it on the screen — [Shadertoy import](docs/vism-shadertoy-import.md) |
| A **Shadertoy** (`mainImage`, or the exported JSON with Buffer A–D and Common) | as above, unchanged | GLSL → WGSL through naga; buffers become host-owned feedback |
| A shader that reads **another time** — its own layer, what is composited beneath it, its group, the whole comp | as above | the host renders that time and hands the picture over; the effect remembers nothing |
| A shader that **feeds back** (`PERSISTENT`) | as above | the host keeps the history and replays from checkpoints, so scrubbing lands on the same picture as playing |

Everything in that table has a test that renders on a real GPU, and a bundled sample on the shelf. Save the file and the window reloads it; a broken save shows its reason and keeps the last good program.

The editing side is an AE-family editor in Flutter: layers, keyframes with easing, groups with Each/Whole effect scope, track mattes, clipping, cameras in 2D / 2.5D / 3D, text, shapes with path effects, audio, mp4 export through one deterministic path. It is a macOS development build, used daily by one person to make music videos. It is not yet a release.

## Why open source

After Effects established much of the language of modern motion graphics. Cavalry and Autograph demonstrate strong alternatives. AviUtl demonstrates how far a lightweight, locally run tool and its extension community can carry a creative culture. Motolii learns from these and does not treat proprietary software as a failed choice. It is open because it favors a future that does not have to converge on one universal host: code, project semantics, tests and rulings stay inspectable and forkable, so different communities can continue the work, disagree with it, or build compatible hosts without asking one owner to define the future for everyone.

A [Vism](docs/vism-package-concept.md) — Motolii's effect package — is not a universal plugin format. It cannot be loaded into After Effects or AviUtl. Motolii is its first host; the portability target is compatible hosts and forks that adopt the public contract.

## Read before you write

- [Concept](docs/concept.md) — what Motolii is and is not
- [Plugin resources and the laws of time](docs/plugin-resources.md) — why an effect never remembers a frame, and how the host does it instead
- [The field model](docs/vism-field-model.md) — one effect on every material
- [Shadertoy import](docs/vism-shadertoy-import.md) — paste, orientation, time references, feedback, multi-tab
- [AE pain points](docs/ae-pain-points.md) — the evidence behind the design
- [Contributing](CONTRIBUTING.md) · [Agent entry](motolii/AGENTS.md) · [Stage 5 workspace](docs/stage5/README.md)

## Build and run — macOS

Install Rust, Flutter with macOS desktop support, and Xcode's macOS build tools. Put Flutter on `PATH` or set `FLUTTER_BIN`. Media operations use FFmpeg (`.cargo/config.toml` names its location; adjust it to your machine). The renderer is a pinned fork of Rerun; today the pin points at a local clone, which is the first thing to fix before anyone else can build this.

```sh
scripts/motolii-ui.sh check
scripts/motolii-ui.sh native       # first run, or after Rust changes
scripts/motolii-ui.sh dev          # start an empty project
```

Open an existing project with File → Open, or `scripts/motolii-ui.sh dev /absolute/path/project.rrd`. Keep the dev process running: `r` or `scripts/motolii-ui.sh reload` for Dart changes; Rust changes need `native` and an app restart. Effects in `motolii/crates/motolii-render/vism/` reload on save.

Tests: `cargo test -p motolii-render --lib -- --test-threads=2` (GPU tests are flaky when fully parallel), `cargo test -p motolii-ui --lib`, and `scripts/motolii-ui.sh test` for the Flutter side.

## Status, honestly

- macOS only; the build is pinned to one machine's paths.
- One deterministic path from preview to export; export is mp4 through FFmpeg.
- Freeze frames, reverse playback of feedback effects, and a per-frame render cache are not done.
- The UI is functional and under active revision; screenshots would be out of date within a week.

The earlier README, written before the effect system existed, is kept as [docs/archive/README-2026-08-27-two-what-ifs.md](docs/archive/README-2026-08-27-two-what-ifs.md).

## License

Dual-licensed under [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option — a permissive, local, forkable core.
