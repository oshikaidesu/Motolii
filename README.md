# Motolii

> **A GPU-native, plugin-first motion-graphics compositor in Rust — built for music-video (MV) creation.**
> _Early stage: building in the open, looking for collaborators._

After Effects is powerful but heavy, and its extension gateway is narrow (C++ SDK, fragile JS expressions, limited plugin UI). This project is a bet on a different foundation: keep pixels resident on the GPU, make the plugin boundary dead-simple (so effects are easy to write — even by an LLM), and stay focused on getting a 3–5 minute MV out the door.

**This is not** a general NLE, a Nuke-style VFX compositor, or a color-grading suite. It is a motion-graphics tool: keyframes, easing, procedural overlays, 2.5D/3D (glTF) + video in one composite, and a single-song MV export.

---

## Status

Pre-1.0, pre-first-release. The engine core is coming together (Rust workspace + CI + golden-image tests); the UI is intentionally not started yet (see roadmap). We are working toward a first demo:

**Current goal (M1 "exit demo"):** render a simple 2-layer composite from a project file — a real video background + a rectangle shape sliding right with easing — out to an mp4, from the CLI. No UI yet.

## Open & local — and staying that way

**No account. No online license. No acquisition rug-pull.** It runs fully offline, and the MIT/Apache-2.0 license means it can always be forked and continued.

This matters right now: several "free" After Effects alternatives went free _via acquisition_ and come with strings — Cavalry (acquired by Canva) requires signing in with a Canva account; Autograph's maker Left Angle wound down entirely (site and all) and it now runs through the Maxon App with account-based license assignment (and dropped Linux). A community already burned by subscriptions is right to be wary.

We take the opposite bet: a permissive, local, no-account tool that can't be taken away or gated later. The honest flip side is sustainability — OSS projects can die from _lack_ of maintainers (see Natron). Our answer has two parts: (1) the license itself — MIT/Apache and fully local, so it can be forked and carried on by anyone; and (2) **LLM-driven development** (see below), which widens the contributor gateway instead of depending on a funded core team.

## LLM-driven development

This project is built to be developed and extended primarily with LLMs, and that is a deliberate sustainability strategy — not an afterthought.

- **The plugin boundary is intentionally simple** (`texture in + params → texture out`, params auto-generate their UI) specifically so that effects and extensions are easy for an LLM to write. Extending the tool shouldn't require learning a heavy C++ SDK or fragile scripting — describe the effect, get a plugin.
- **Development is spec-driven and self-verifying**: each task has an automatic pass/fail (`cargo test` / golden images), which is exactly what makes autonomous/LLM-assisted contribution safe and parallelizable. See [`AGENTS.md`](AGENTS.md) and [`docs/specs/`](docs/specs/).
- **Why this de-risks funding**: the classic OSS failure is "the developer(s) ran out of time/money." By lowering the barrier to author plugins and core changes with an LLM, the _developer_ gateway widens the same way we widen the _user_ gateway — the project doesn't hinge on funding a dedicated dev team to stay alive and growing.

## Coming from the Japanese scene (AviUtl)

I'm a creator from the Japanese sphere, where the de-facto free After Effects alternative isn't Cavalry or Autograph — it's **AviUtl**, a beloved, community-sustained free tool that has been a backbone of Japanese motion-graphics / MV / vtuber work for well over a decade. In 2025 it was reborn as a from-scratch 64-bit rewrite, **AviUtl2 (ExEdit2)**.

AviUtl is living proof of the thesis behind this project: a free, local, plugin/script-extensible tool, kept alive by its community, can outlast commercial churn. That spirit is exactly what we build on. But it also shows the **limit** of that model — and that limit is the crux everything snags on:

- **It isn't actually open source (the crux).** AviUtl/AviUtl2 ship a *Plugin SDK*, but the **core is closed and single-author**. The community can bolt plugins on the outside; it can never fix or advance the core. The proof: after v1.10 in 2019 the core sat **frozen for ~6 years** until the 2025 rewrite — people wanted improvements but couldn't touch it. AviUtl2 repeats the same single-author, closed-core pattern. This is the one thing plugins can never solve, and it's exactly what an **MIT/Apache, forkable core** is for: if we stop, anyone can continue it.

And even setting that aside, AviUtl2 (beta50, 2026) leaves gaps this project targets — and yes, I know these firsthand:

- **Windows-only** (Win10 64-bit; requires AVX2 + DirectX 11.3 + an ROV-capable GPU) — no macOS/Linux, and older hardware is locked out. Our Rust/wgpu core is cross-platform _by design_ (Vulkan/Metal/DX12); v1 targets one OS first, but the architecture isn't Win32/DirectX-bound.
- **No real 3D compositing** — native support is static OBJ only (no camera/lights/bones); its own docs recommend rendering serious 3D in Blender and importing. We put glTF meshes and video in one composite.
- **Whole-frame caching** (AE-style), not selective caching — we cache by node × time-range × params, so a small change doesn't re-render everything.
- **No analysis-driven generative** (frame-wide color/tracking → parameters). AviUtl's tracking is point/region-based via plugins. We reserve this and ship it in a final phase.
- **Plugin authoring** is a native C ABI / Lua scripts, and the ecosystem was reset (32-bit plugins don't carry over). We aim for a dead-simple, LLM-writable plugin boundary instead.
- Still **beta / occasionally unstable**, with a largely Japanese-centric ecosystem.

None of this is a knock on AviUtl — it's a remarkable project and a direct inspiration. It's a map of where a modern, **open**, GPU-native, cross-platform-capable, LLM-extensible foundation can go next. The single biggest difference we insist on: the **core is open and forkable**, so it can never be frozen behind one person again.

## Why this architecture

- **Lightness is structural, not a tweak.** Pixels live in GPU (wgpu) textures and are never bounced to the CPU. The dominant cost in compositing is memory bandwidth × round-trips, not resolution — so we minimize round-trips by construction. See [`docs/performance-model.md`](docs/performance-model.md).
- **Plugin-first, low barrier.** Effects are a simple `texture in + params → texture out` boundary; parameter panels are auto-generated from the plugin's declared params. The narrow, expensive plugin gateway is exactly what we think drives most feature complaints about AE — see [`docs/ae-pain-points.md`](docs/ae-pain-points.md).
- **Deterministic core.** `render_frame(t, Quality)` is a pure function; preview and export go through the same path (no "looks different when exported" bug). Document state is a single serde data structure edited by commands (undo/journal for free), read-only snapshots for rendering.
- **Learn from the graveyard.** Design decisions are cross-checked against why similar projects stalled (Olive rewrite, Natron's cache/threading deadlocks). See the pitfall catalog in [`docs/pitfalls-and-roadmap.md`](docs/pitfalls-and-roadmap.md).

## Tech stack (decided)

| Layer | Choice |
|---|---|
| Render core | Rust + [wgpu](https://github.com/gfx-rs/wgpu) (VRAM-resident) |
| UI (planned) | [Slint](https://slint.dev) (wgpu zero-copy texture embedding, Japanese IME) — not started yet |
| Decode / encode | ffmpeg as a sidecar process (raw YUV in, mp4 out) |
| Structure | Cargo workspace (`crates/motolii-*`) |

## Design docs (start here)

The design is documented in depth — this is the fastest way to understand the direction and to find where to help.

- [`docs/README.md`](docs/README.md) — reading guide & glossary
- [`docs/concept.md`](docs/concept.md) — what it is / isn't, the decision ledger
- [`docs/pitfalls-and-roadmap.md`](docs/pitfalls-and-roadmap.md) — pitfall catalog + roadmap (M0–M5) + freeze gate
- [`docs/performance-model.md`](docs/performance-model.md) — why it can be lighter than AE

## Build & run (early)

Requires a recent Rust toolchain (pinned in `rust-toolchain.toml`), `ffmpeg`/`ffprobe` (v6+), and a Vulkan/Metal/DX12 GPU (CI runs on software Vulkan / lavapipe).

```sh
cargo test --workspace
# Render a project to mp4 (see docs/specs/M1-vertical-slice.md for the JSON schema)
cargo run -p motolii-cli -- export-project path/to/project.json
```

## Looking for collaborators

This is designed for parallel, spec-driven, **LLM-assisted** development (each task has an automatic pass/fail via `cargo test` / golden images, so an LLM can implement a ticket and prove it). If you like Rust + GPU + motion graphics — or just want to point an LLM at a good-first-issue — there is a clear on-ramp:

- Read [`docs/README.md`](docs/README.md), then the milestone specs under [`docs/specs/`](docs/specs/).
- Contributor conventions and the absolute rules live in [`AGENTS.md`](AGENTS.md) (applies to human and AI contributors alike).
- Good areas to help early: render nodes & shapes, golden-image test coverage, ffmpeg/decoder robustness, docs. (The UI is deliberately later and community-driven.)

Issues / discussion: please open a GitHub issue. (A `good first issue` set is being curated from the backlog.)

## License

Licensed under either of

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT license ([`LICENSE-MIT`](LICENSE-MIT))

at your option. This permissive dual license (the Rust ecosystem standard) keeps the ecosystem free to build on — including selling commercial plugins.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

**Dependency note:** third-party dependencies carry their own licenses. In particular Slint (planned UI) is used under its royalty-free desktop license (attribution required; verify terms before distribution), and ffmpeg is invoked as an external process (not linked). See [`docs/references.md`](docs/references.md).
