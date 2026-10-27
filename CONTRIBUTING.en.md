# Contributing

[日本語](CONTRIBUTING.md)

## What you can do

- **Issues**: bugs, a build that stopped, or "I want to use this look" (a URL and what is good about it). There are templates. English or Japanese.
- **Pull requests**: welcome. Motolii is dual licensed under Apache-2.0 or MIT; unless you explicitly state otherwise, what you send is under the same two licences.
- **Authors of bundled works**: if you want the way your work is bundled changed, or it removed, open an issue and it will be done ([`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)).

Please follow the [Code of Conduct](CODE_OF_CONDUCT.md). Security problems go to the Security tab, not to issues ([`SECURITY.md`](SECURITY.md)).

## Building

As in [`README.en.md`](README.en.md) ("Try it"). If you will commit, once in your clone:

```bash
git config core.hooksPath .githooks
```

How the code is laid out: [`ARCHITECTURE.md`](ARCHITECTURE.md) and [`filament/README.md`](filament/README.md) (which files the current build uses; the rest are earlier probes).

## Checks

| What | Command | When it holds |
|---|---|---|
| The ledger (rule 2 below) | `python3 .githooks/assets_check.py` | `ASSETS.md covers all … files` |
| The engine's checks, together (they need Python's Pillow and numpy: `python3 -m pip install --user pillow numpy`) | `filament/check/all.sh` | `all checks held` (exit 1 if any says NO); one at a time: `filament/check/*.sh` |
| The window's tests | `cd skin/book && flutter test` | Tests that need the engine are skipped if it is not built |
| The window's code | `cd skin/book && flutter analyze lib test integration_test` | No errors (97 notices today) |
| The first piece in the window | `cd skin/book && flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/first_piece_test.dart` | Writes `docs/media/window_step1..4.png` |

GitHub Actions runs the ledger check on every push and pull request (`.github/workflows/assets.yml`; it only reports, it does not block a pull request). The whole build and the checks run in `.github/workflows/build.yml` (on an Apple silicon Mac, built as the README says, then `check/all.sh`, `flutter analyze`, `flutter test`; on pull requests that touch the code, and by hand from the Actions tab).

## Rules

1. **Look before you write.** Search [`ASSETS.md`](ASSETS.md) (the ledger of what exists) and existing implementations on the net. If something can be used the way its author meant, use it.
2. **A new file gets a line in the ledger.** `ASSETS.md` says what exists and how far it was checked. The pre-commit hook stops a commit whose files have no place there. Add a row, or add the file's place to an existing row (places in backticks; a folder covers everything in it). **If that is too much, commit with `git commit --no-verify` and say so in the pull request; the maintainer will add it when merging.** The ledger is in Japanese; English in your row is fine.
3. **Say where things came from.** In the commit or pull request: what was borrowed from where (URL, version or commit), what was checked and what is not done yet. Example: `git show d46f69cac`. Keep the first line (the subject) to about 72 characters and put the details after a blank line (git's convention; much of the history so far has long subjects, but from now on).
4. **Bundling other people's work.**
   - Keep the author's folder as unchanged as possible, with their licence text (`LICENSE`) and where it came from (`SOURCE.txt`: name and author / licence / from / page) beside it.
   - Work whose author publishes no licence text is not bundled; fetch it from the author at build time instead (`filament/midi_roll.sh` is an example).
   - Changes to upstream code are kept as patches, applied to a pinned upstream at build time (`filament/dawn_probe/tvg_on_dawn.sh` is an example).
   - Add a row to [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

## Good first tasks

| What | Where to start | Difficulty |
|---|---|---|
| An `.app` you can hand out: put the Homebrew libraries the engine loads (GStreamer, glib, SQLite, LuaJIT, QuickJS, and ffmpeg for writing) inside the `.app`, with [dylibbundler](https://github.com/auriamg/macdylibbundler) or GStreamer's macOS framework | `filament/edit.sh`, `skin/book/macos/` | Medium |
| AviUtl2 calls still missing (`obj.drawpoly`, the "circle" figure, …), following the references (AviUtl2's lua.txt, sashimono-edit) | `reports/aviutl2/converter_plan.md`, `filament/aviutl.cpp` | Medium–hard |
| TSL looks on the editor's shelves | `filament/tsl/`, `reports/tsl_collection.md` | Medium |
| Inside the app, a screen effect (ISF, WGSL) put on a layer shows the composition white (the written video is right; `drop_effect_test.dart` shows it) | `filament/live.h`, `filament/thing_fx.h` | Medium–hard |
| Fix the three failing window tests (see Known gaps) so the whole suite passes run together | `skin/book/test/` | Medium |
| English beside the headings of `ASSETS.md` | `ASSETS.md` | Easy |
| The window's integration tests write into `reports/`; make them use a temporary folder | `skin/book/integration_test/` | Easy |

## Known gaps

- macOS on Apple silicon only.
- The window's tests (`flutter test`) have failures when run together (on 2026-10-27, 125 of 150 pass on the author's Mac and 117 of 126 on the CI Mac). Most pass one at a time; three fail alone: Parent (the position after a parent is given), the anime.js shelf (its samples were removed on 10/26 and the test was not updated), and the Posterize and Boxinator knobs. Fixing them is a good first task. In CI that step only reports.
- The engine (`libmotolii.dylib`) loads Homebrew's GStreamer and others, so a built `.app` does not run on another Mac as it is.
- `filament/` mixes the current engine with earlier probes at one level; [`filament/README.md`](filament/README.md) lists which files the current build uses.
