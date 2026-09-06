# Contributing to Motolii

Current development is **Stage 5**, in `motolii/`. Flutter UI work belongs in `motolii/ui`; the repository-root Cargo workspace defaults to the Flutter native bridge.

## Find the owner before editing

1. Read [the concept](docs/concept.md) and [adopted UI behavior](docs/stage5/product-contract.md).
2. Use the [Stage 5 responsibility table](docs/stage5/README.md) to find the code that owns your change.
3. Search [decision-index](docs/decision-index.md) for relevant prior decisions. Historical entries are evidence, not a reason to override newer user decisions.
4. Agent instructions are in [motolii/AGENTS.md](motolii/AGENTS.md).

Document/Intent owns authored changes and Undo. Flutter owns presentation and transient interaction state. Reuse the canonical doc/render crates rather than copying them into UI. The current Rust editor layer is `motolii/ui/native/src/editor`; old Dioxus helpers are historical references, not a second maintenance target. [Module boundaries](docs/stage5/modules.md) and [provenance](docs/stage5/imported-edit-helpers.json) identify the owners.

## Development loop

Use the prerequisites in [README](README.md), then run from the repository root:

```sh
scripts/motolii-ui.sh check
scripts/motolii-ui.sh native
scripts/motolii-ui.sh dev
```

For Dart edits, keep the application running and use `scripts/motolii-ui.sh reload` or terminal `r`. Rebuild the native library only when its Rust code changes. Save before restarting the application. Distribution packaging is unfinished; legacy bundle scripts are not the Stage 5 release process.

## Verification

Use checks appropriate to the change:

```sh
# Current entry points, documentation links, and core ownership.
scripts/motolii-ui.sh check
# Whitespace errors.
git diff --check
# Combined suite for changes spanning Stage 5 boundaries.
scripts/motolii-ui.sh test
```

Individual Rust targets run from the repository root with an explicit `-p` package; Flutter tests run from `motolii/ui/`. The [technical evidence](docs/stage5/technical-boundaries.md) separates component tests, comparison-port observations, and real-window acceptance. Report commands and results honestly; a skipped test is not a pass, and a test pass is not a visual review.

`scripts/check-docs.sh` remains the wider historical documentation audit. Its old inventories and historical links are distinct from the Stage 5 entry check; do not use a successful local check to claim the entire historical tree has been audited.

## Issues and pull requests

Describe the concrete problem, how to reproduce it, the owner of the change, and the resulting behavior. Keep scope reviewable. The existing [issue template](.github/ISSUE_TEMPLATE/closed-contract.yml) and [PR template](.github/pull_request_template.md) remain available; consult [CODEOWNERS](.github/CODEOWNERS) for ownership rules.

Update affected current documents when meaning changes. Preserve unrelated local edits. Do not use old paths from historical documents as current targets, duplicate core ownership, hide incomplete features behind completion claims, or add a second rendering implementation to imitate the current result.

The [previous contributor guide](docs/stage5/history/contributing-before-entry-cleanup.md) is retained for history. Current agent policy comes from `motolii/AGENTS.md`, not old guide text.

## License

Contributions are dual-licensed under [Apache-2.0](LICENSE-APACHE) and [MIT](LICENSE-MIT), unless explicitly stated otherwise.
