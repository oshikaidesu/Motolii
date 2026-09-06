# Motolii

**A layer-based video editor and compositor: Lottie's editing model, Rerun's rendering foundation, Ableton-inspired organization, and direct manipulation.**

文字・画像・動画・立体・点群を同じStageで扱い、普通の操作で快適に映像を作るソフトを目指しています。守るのはユーザーの意図であり、仮実装や過去のUI構造ではありません。

- [現行コンセプト / Current concept](docs/concept.md)
- [UIと操作の採用事項](docs/stage5/product-contract.md)
- [Stage 5の構成・残作業](docs/stage5/README.md)
- [Contributor guide](CONTRIBUTING.md) · [Agent entry](motolii/AGENTS.md)

## Current status

**Stage 5: Flutter is the primary UI and development route.** The shared Rust core and UI module boundaries are integrated. Remaining panel refinements, full payload typing and release packaging are ongoing product work. This is a macOS development application, not a claim of full feature parity or cross-platform release readiness.

| Responsibility | Current source |
|---|---|
| Flutter UI | [`motolii/ui`](motolii/ui) |
| Native UI bridge | [`motolii/ui/native`](motolii/ui/native) |
| Document, hierarchy, keyframes, Undo | [`motolii/crates/motolii-doc`](motolii/crates/motolii-doc) |
| Evaluation, rendering, export | [`motolii/crates/motolii-render`](motolii/crates/motolii-render) |
| Workspace status and known unfinished work | [`workspace.json`](docs/stage5/workspace.json) |

Lottie is the underlying editing model; projects currently save through Rust Document's `.rrd` path. On macOS, the renderer hands its output to Flutter through a shared Metal/IOSurface texture. See [technical boundaries and evidence](docs/stage5/technical-boundaries.md) for what has and has not been verified.

## Build and run — macOS

Install Rust, Flutter with macOS desktop support, and Xcode's macOS build tools. Put Flutter on `PATH` or set `FLUTTER_BIN` to its executable. `.tools/flutter` is an optional ignored local SDK link. Media operations use FFmpeg; see the [development guide](docs/stage5/README.md).

From this repository root:

```sh
scripts/motolii-ui.sh check
scripts/motolii-ui.sh native       # First run, or after Rust changes
scripts/motolii-ui.sh dev          # Start an empty project
```

To open an existing project, use File → Open, or `scripts/motolii-ui.sh dev /absolute/path/project.rrd`.

Keep the dev process running. Use `r` in its terminal or `scripts/motolii-ui.sh reload` for Dart changes. Rust changes need a native build and application restart after saving. `R` resets UI state and is not the normal edit loop.

## Verify the change you made

- Documentation and workspace routing: `scripts/motolii-ui.sh check` and `git diff --check`.
- UI appearance: hot reload and inspect the real window.
- Editing behavior: run the relevant test target, then check the actual operation. `scripts/motolii-ui.sh test` is the combined Stage 5 suite; do not rerun it for every spacing edit.

A successful build is not proof of usable UI. Record the actual checks and distinguish untested behavior from verified results.

## Historical implementations

`motolii/src/ui` is the legacy Dioxus/Blitz UI. `app/`, `next/`, and the root `crates/` contain earlier generations. They are references, not alternate current startup instructions. Start new UI work in `motolii/ui`.

The [previous mixed-generation README](docs/stage5/history/readme-before-entry-cleanup.md), [vision](VISION.ja.md), [manifesto](MANIFESTO.ja.md), and [document recovery map](docs/stage5/document-map.md) preserve background and provenance. Their historical commands and completion claims do not override Stage 5.

## About the name

**Motolii** (モトリー) comes from *motley*: varied parts forming one whole. The spelling passes through a Japanese sound; the final `lii` is intentional.

## License

Dual-licensed under [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option. Third-party dependencies retain their own licenses; consult [references](docs/references.md) before distribution.
