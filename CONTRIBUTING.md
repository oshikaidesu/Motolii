# Contributing to Motolii

Motolii is a pre-1.0, specification-driven Rust compositor maintained by a single owner. Contributions are welcome in rendering, document semantics, tests, tooling, UI, plugins, documentation, and prior-art review.

This project keeps its design decisions in the repository rather than in conversations. Most review friction comes from changes that silently contradict a decision that was already made and written down, so the reading step below is not optional.

## Read this before you write code

| Read | Why |
|---|---|
| [`README.md`](README.md) | What Motolii is, its scope, and its non-goals |
| [`VISION.ja.md`](VISION.ja.md) | Problem statement and long-term direction (Japanese) |
| [`docs/README.md`](docs/README.md) | Reading order, glossary, and the file map for `docs/` |
| [`docs/decision-index.md`](docs/decision-index.md) | Reverse index: search it by topic keyword before touching that topic |

Most design documents under `docs/` are written in Japanese. `README.md`, this file, and the GitHub issue/PR templates are English.

## Build

Requirements:

- the Rust toolchain pinned in [`rust-toolchain.toml`](rust-toolchain.toml);
- `ffmpeg` and `ffprobe` 6 or later;
- Vulkan, Metal, or DX12 graphics support.

```sh
# Build the workspace.
cargo build --workspace

# Render a project to mp4 through the real export path.
cargo run -p motolii-cli -- export-project path/to/project.json
```

`scripts/setup-local-deps.sh` provisions a local ffmpeg environment if you do not have one on `PATH`.

For the macOS desktop application bundle, use [`scripts/build-macos-app.sh`](scripts/build-macos-app.sh) rather than assembling a bundle by hand.

## Verify — there is no CI

**This repository has no continuous integration.** The GitHub Actions connection was retired on 2026-08-09 by an explicit decision ([`docs/reviews/2026-07-31-repository-validation-topology-decision.md`](docs/reviews/2026-07-31-repository-validation-topology-decision.md), section 5). No check runs on your pull request, and no remote result is accepted as completion evidence.

Verification is your responsibility and runs locally. Run the commands below, then paste the **exact commands and their actual results** into the PR body.

```sh
# Portable local profile: docs, then Rust fmt / clippy / locked workspace tests.
./scripts/validate.sh local

# Required whenever you change docs/.
./scripts/check-docs.sh

# The Rust lane.
cargo test --locked --workspace

# Whitespace errors in the diff.
git diff --check
```

`./scripts/validate.sh --list` shows the other lanes (`policy`, `tooling`, `web-build`, `web-contract`, `web-visual`). `./scripts/test-local.sh` is the same local profile after `scripts/setup-local-deps.sh` has provided the ffmpeg environment.

Rules that make the evidence meaningful:

- `cargo test` proves the **Rust lane only**. It does not stand in for docs, product end-to-end behavior, real hardware, or human judgment.
- Do not report a lane as passing if it did not actually run. A skipped, interrupted, or zero-test run is not a pass.
- Do not edit tests, goldens, thresholds, or lint suppressions to turn a red run green. If a test looks wrong, stop and raise that as a separate question.
- Report a partial result as partial. Separate what passed, what failed, and what you did not run.

## Opening an issue

Use the **Closed contract** issue template (`.github/ISSUE_TEMPLATE/closed-contract.yml`). One issue is one contract boundary that will produce one commit.

Fill in the capsule with actual current values — target paths, the owner, the oracle command that would fail if the change were wrong. Do not guess at a target or an owner that you have not confirmed exists in the code. If you cannot close the contract because something is genuinely missing, say so: where you searched, what candidates you found, why they did not fit, and the exact gap. That is a useful issue too.

Bug reports and design questions are welcome in the same template; put the observed behavior and the reproducing command in the capsule.

## Opening a pull request

- One PR = one contract boundary = one commit. Keep the diff inside the allowlist declared in the issue.
- Fill in [`.github/pull_request_template.md`](.github/pull_request_template.md) completely. The Evidence section is where your local command output goes.
- Small, independently verifiable pull requests are strongly preferred over large ones.
- A PR is a **candidate** until the owner adopts and integrates it. Merge is not implied by a clean diff or a green local run.

If your change modifies a specification or a decision, update `docs/decision-index.md` and any affected ledger **in the same commit**. A decision that exists only in a PR comment does not exist.

## Review

Review goes through a single owner (`@oshikaidesu`). See [`.github/CODEOWNERS`](.github/CODEOWNERS): the single-owner policy is deliberate, and no second code owner is appointed. Protected test assets — goldens, golden policy, CPU reference implementations, and tolerance code under `crates/motolii-testkit/` — require owner approval, and changes to them are held to a higher bar than ordinary code.

Practical consequence: review is serialized through one person. Small, self-contained PRs with complete local evidence move; large or under-evidenced ones wait.

## What not to do

- **Do not reverse a settled decision implicitly.** If your change contradicts a decision recorded in `docs/`, quote the exact file and passage in the issue and PR, and get agreement before implementing. Silently reintroducing a rejected approach is the single most expensive mistake here.
- **Do not add generic helpers or speculative abstraction.** No "for future use" traits, config knobs, plugin hooks, or generic frameworks. Search for an existing equivalent first; test helpers belong in `motolii-testkit` rather than being duplicated.
- **Do not widen scope from a finding.** Fix what the contract covers; report the rest.
- **Do not put pixel processing on the CPU, split preview and export into separate render paths, give plugins hidden mutable state, or expose vendor/OS GPU APIs through the plugin contract.** These product contracts are documented in [`docs/README.md`](docs/README.md).
- **Do not rely on remote checks.** There are none.

Code comments are written in Japanese and explain *why*, not *what*.

## License

By contributing you agree that your contribution is dual-licensed under Apache License 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE)) and MIT ([`LICENSE-MIT`](LICENSE-MIT)), at the user's option, unless you explicitly state otherwise.
