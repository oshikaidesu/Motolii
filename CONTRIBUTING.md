# Contributing to Motolii

Current development is **Stage 5**, in `motolii/`. Flutter UI work belongs in `motolii/ui`; the repository-root Cargo workspace defaults to the Flutter native bridge.

## Find the owner before editing

1. Read [the concept](docs/concept.md) and [adopted UI behavior](docs/stage5/product-contract.md).
2. Use the [module responsibility table](docs/stage5/modules.md) to find the code that owns your change.
3. Search [decision-index](docs/decision-index.md) for relevant prior decisions. Historical entries are evidence, not a reason to override newer user decisions.
4. Agent instructions are in [motolii/AGENTS.md](motolii/AGENTS.md).

Document/Intent owns authored changes and Undo. Native ViewerState owns selection, viewing time and observer state; Flutter owns widget presentation, focus and layout. Neither is a second owner of the work. Reuse the canonical doc/render crates rather than copying them into UI. The current Rust editor layer is `motolii/ui/native/src/editor`; old Dioxus helpers are historical references, not a second maintenance target. [Module boundaries](docs/stage5/modules.md) and [provenance](docs/stage5/imported-edit-helpers.json) identify the owners.

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

For a focused change, start with its owner's tests (use the README's native dependency environment for Cargo):

| Change | Focused check |
|---|---|
| Document edits and preview projection | `cargo test -p motolii-doc --lib --test edit_transactions` |
| Read-only recording and renderer build boundary | `scripts/motolii-ui.sh check-read-only` |
| Viewing-state ownership | `cargo test -p motolii-ui --lib viewer::tests` |
| Native window attachment lifecycle | `scripts/motolii-ui.sh test-window` |
| Script execution without the editor or GPU | `cargo test -p motolii-script` |

From `motolii/ui`, `flutter test test/window_attachment_test.dart test/session_reconnect_test.dart test/workspace_host_contract_test.dart` checks UI detach/reconnect and the dock's actual WidgetsApp host contract.

Add regression cases at the owning boundary. Do not require a full application or unrelated services to test a pure contract; use actual-window acceptance when behavior depends on window interaction.

`motolii-doc` enables `editing` by default for editor compatibility. Read-only consumers use `default-features = false` and `Recording::load().view()`; production `motolii-render` uses this configuration. Its test fixtures opt into editing only as a dev-dependency. Cargo features are additive, so an editor workspace build still includes editing; this is a dependency boundary, not a sandbox. Keep UI-only work on hot reload, and use focused checks instead of rebuilding every feature configuration after every change. Build-time improvement has not yet been measured.

`scripts/check-docs.sh` remains the wider historical documentation audit. Its old inventories and historical links are distinct from the Stage 5 entry check; do not use a successful local check to claim the entire historical tree has been audited.

## Issues and pull requests

Describe the concrete problem, how to reproduce it, the owner of the change, and the resulting behavior. Keep scope reviewable. The existing [issue template](.github/ISSUE_TEMPLATE/closed-contract.yml) and [PR template](.github/pull_request_template.md) remain available; consult [CODEOWNERS](.github/CODEOWNERS) for ownership rules.

Update affected current documents when meaning changes. Preserve unrelated local edits. Do not use old paths from historical documents as current targets, duplicate core ownership, hide incomplete features behind completion claims, or add a second rendering implementation to imitate the current result.

The [previous contributor guide](docs/stage5/history/contributing-before-entry-cleanup.md) is retained for history. Current agent policy comes from `motolii/AGENTS.md`, not old guide text.

## License

Contributions are dual-licensed under [Apache-2.0](LICENSE-APACHE) and [MIT](LICENSE-MIT), unless explicitly stated otherwise.
