# External police spike (2026-09-30)

Verdict: **Semgrep ADOPT (4 rules), cargo-deny ADOPT (`sources` only).** CodSpeed/Bencher not now.
Run: `scripts/validate.sh police` (`scripts/check-semgrep.sh [--history]`, `scripts/check-deny.sh`). They add to, never replace,
`check-stage5.py`, `reference/owned-budget.tsv` and `check-hygiene.sh`.

## Semgrep — each rule was made red on the commit that had the bug and green on the fix (`semgrep/history.tsv`)

| rule | red → green | hits |
|---|---|---|
| `motolii-swift-status-as-dictionary` | 27680b29e → d51f1a59e (status/render/catalog parsed into a Dictionary in Swift) | 7 → 0 |
| `motolii-native-host-never-waits-the-gpu` | 726d4f480^ → 726d4f480 (host blocked on the GPU inside render) | 1 → 0 |
| `motolii-rust-unbounded-gpu-wait` | 46a5ee34b^ → 46a5ee34b (waits without a timeout) | 9 → 0 |
| `motolii-rust-raw-gpu-infrastructure` | new hits in 39146f7fe over its parent (compute pipeline + shader module + bind-group layout, ceiling 0) | 3 new |

Rule tests: `semgrep --test semgrep/` (fixtures `semgrep/motolii.{swift,dart,rs}`), 5/5. Current tree: silent, except the raw-GPU rule,
which is judged against a baseline because the tree already exceeds the ceiling (`selection_bounds.rs`, `block_program/passes.rs`).

Facts learned about Semgrep here (why the rules look the way they do):
- `pattern-not-inside: impl Drop for $T { ... }` silently excludes the whole file — `fn drop(&mut self)` works. `#[cfg(test)] mod` exclusion did not work either; tests are excluded by path.
- Swift: the typed pattern `-> [String: Any]` matches the real file and misses the same function alone; the rule is a signature regex.
- Path filters are relative to the scan root (scan `.`), and `--test` ignores them.
- `--baseline-commit` follows no file moves: against `origin/main` the block-program code moved by 22e899ed6 shows as 3 "new" hits. CI therefore baselines on the push's previous commit (`github.event.before`).

Not adopted, by the same test:
- **Direct-manipulation post-frame hop** (da59b6f15): the fixed code keeps the same legitimate `addPostFrameCallback` in `panels/stage/window.dart` (build-time changes); the bug was a missing event-time sync. Not path-expressible → TEST (`latency_test`, stage-window `lag` PROBE).
- **MethodChannel outside bridge**: no commit ever violated it and `check-stage5.py` already checks it — it fails the history test, so no rule.
- **Duplicate manual direct queue** (`_pending`/`_running`/while-drain): would be a textual heuristic with many false positives; `commandDirect` call sites cannot be told from intent statically → TEST (`latency_test` ordering).

## cargo-deny

`deny.toml` enforces `sources`: crates.io plus exactly `oshikaidesu/rerun` and `oshikaidesu/re_mp4` (read from `Cargo.lock`: 38 + 1 git crates, 701 registry crates). Red: the same config minus `re_mp4` fails (exit 8).
Observed, not enforced: `bans` ok; `licenses` fails unconfigured; `advisories` reports vulnerabilities (quick-xml, rustls-related) and unmaintained crates (`paste`, `ttf-parser`) — a dependency-upgrade task, out of scope.

## Who watches what

| law / failure | existing guard | Semgrep | cargo-deny | behavioural test |
|---|---|---|---|---|
| status parsed in Swift | none | **rule** | | latency probe (94 layers) |
| direct post-frame hop | none | rejected | | `latency_test` lag |
| raw GPU pipeline creation | `owned_budget` test (red at HEAD, not in CI) | **rule (baseline)** | | |
| MethodChannel outside bridge | `check-stage5.py` | rejected (no past bug, duplicate) | | |
| unrelated rebuild | none | no | | `latency_test` slice counts |
| preview-only property | none | no | | `one_layer_moving_reprints_one_row` |
| direct queue ordering | none | no | | `latency_test` |
| hidden Camera render | none | no | | `latency_test` render time |
| blocking GPU wait | `owned_budget` (red at HEAD) | **2 rules** | | `frame_cost_probe` |
| unknown git dependency | `check-stage5.py` (paths only) | | **sources** | |

Two internal guards are red at HEAD and were not run by CI: `owned_budget` (6 patterns over ceiling) and `check-stage5.py` (hidden process state in `store/ids.rs`, taffy named in `frame_graph/flow_program.rs`).

## Performance services (not adopted)

Headless probes that suit a continuous benchmark later: `build_status` at 4 and 94 layers, FrameGraph evaluation, status encode. GUI pointer→visible, Flutter frames and GPU contention are flaky on shared CI, so they are out.
