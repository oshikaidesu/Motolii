# Retired source

The current editor has one Cargo workspace at the repository root and one Flutter application in `motolii/ui`. Document and rendering code live in `motolii/crates`.

Old hosts and probes were removed from the working tree after preserving them in public commit [`fb8818db`](https://github.com/oshikaidesu/Motolii/tree/fb8818db514c8de028e757109f5751870a1f3447). This is the recovery point, not an alternative development entry.

The preserved paths are `app/`, root `crates/`, `next/`, root `ui/`, `spikes/`, and the old Dioxus host's `motolii/src/`, `motolii/tests/`, `motolii/vendor/`, `motolii/.cargo/`, and `motolii/Cargo.toml`. Its unused dynamic-link bridge `motolii/crates/motolii-road/` was also retired; the ownership-budget test remains under `motolii-doc/tests`.

Use `git show fb8818db:<path>` to inspect an old file. Historical document links point to that immutable version. Build outputs and user documents were not removed. Do not restore a legacy host to fix the current editor; port a needed contract into its current owner with a regression test.
