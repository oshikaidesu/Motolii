# D1l `new_v1` lint / protected semantic test conflict decision (2026-07-16)

Status: **Decision amendment — specification correction before PR #200 repair**.

## Conflict

The current D1l constructor decision simultaneously requires:

1. `Document::new_v1()` to carry Rust's `#[deprecated]` attribute;
2. `cargo clippy --workspace --all-targets -- -D warnings` to pass;
3. existing semantic test files to remain byte-for-byte unchanged.

`crates/motolii-doc/tests/d1i3_blend_mode.rs` and `d1i3_lookat_follow.rs` are protected semantic files and already call `Document::new_v1()`. With `--all-targets`, the deprecation warning is emitted at those call sites and becomes an error under `-D warnings`. Adding `#![allow(deprecated)]` changes protected files and is rejected by D1i-4. Replacing the constructor changes protected fixtures. A workspace lint suppression would hide the warning everywhere.

The three requirements are therefore structurally incompatible; this is not an implementation inconvenience.

## Decision

Rust's `#[deprecated]` attribute is **not** applied to `Document::new_v1()`. The API remains public and keeps the exact v1 semantics required by integration, migration, and compatibility fixtures. It is marked `#[doc(hidden)]` and documented in source as legacy-fixture-only.

The existing AST-based policy test is the enforcing judge, because it is stronger than a warning: it rejects every non-test workspace `src` call except the closed `migrate.rs` legacy allowlist. Its negative fixtures remain mandatory. Product construction still has exactly one supported entry, `Document::new_current()`.

This amendment changes no Document version, serialized bytes, migration, constructor return value, Command behavior, or pixel meaning.

## PR #200 repair order after this decision reaches main

1. Remove the `#[deprecated]` attribute from `new_v1`; add only `#[doc(hidden)]` and the legacy-only source documentation.
2. Remove PR-added `allow(deprecated)` suppressions. The two protected semantic files must have an empty final diff against `origin/main`; do not touch their comments, imports, fixtures, or expectations.
3. Keep the PR #200 correction that uses `Document::new_current()` in the non-test `journal/project.rs::inject_unapplicable_committed_edit` helper. It creates a current-format test project and is not a legacy fixture. Do not add that file to the `new_v1` allowlist; any other non-test `src` call must be moved behind a test boundary, accept an explicit Document fixture, or use `new_current()` only when its semantics are genuinely current-project creation.
4. Keep and run `d1l_new_v1_allowlist_policy`, including its negative cases. Do not weaken its path/cfg analysis or allowlist.
5. Run, in order:
   - `./scripts/check-golden-update-policy.sh`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test -p motolii-doc`
   - `cargo test --workspace`
6. If any gate requires a protected-file change, global lint suppression, or a new constructor alias, stop and amend the decision rather than adding a special-case.

The PR branch may contain a narrow review fixup, but main must receive one ticket commit through squash merge.

## Anti-hack guard

- no `allow`/`expect`/`ignore` added to semantic files;
- no classification/policy/CI/RUSTFLAGS relaxation;
- no `new_v1`→`new_current` substitution in legacy fixtures;
- no `cfg(clippy)` or target-specific deprecation trick;
- no product code call to `new_v1`;
- no version/min-reader mutation outside the existing D1l contract.

## Completion judgment

1. Every path classified `semantic` in `classification.tsv` has zero diff from main and the golden policy passes.
2. The AST gate rejects a newly injected non-test `src` call and accepts only its documented migration/test closure.
3. `new_current()` remains v4/read-write; `new_v1()` remains v1/legacy; their roundtrip/migration tests are unchanged.
4. Clippy with `-D warnings` and the full workspace tests are green without deprecation suppression.
