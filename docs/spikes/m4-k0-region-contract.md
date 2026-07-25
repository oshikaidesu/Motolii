# M4-K0 spatial region contract spike (test-only)

## Base

- **GRAIN**: K0
- **BASE_SHA**: `eb3ed25572603fb7b2a982fd99c08bc6ffce92e1` (`refs/heads/codex/m4-k0-composer-fallback-20260725`)
- **Implementer**: user-authorized `composer-2.5` fallback after Spark context stop; v4 oracle repair executed by `gpt-5.3-codex-spark` under `claude-opus-5` order.
- **Quarantine**: v1/v2 K0 worktrees and branches were **not** read, diffed, copied, or used as implementation input.

## Artifact

| Path | Role |
|---|---|
| `crates/motolii-render/tests/k0_region_contract.rs` | Private integration-test-only model + exactly 15 tests |
| `docs/spikes/m4-k0-region-contract.md` | This freeze report |

No `crates/*/src/`, `Cargo.toml`, public API, schema, or GPU paths were touched.

## K0 completion conditions → tests

| # | M4-K0 condition (spec) | Test name |
|---|---|---|
| 1 | Blur expands input region by declared radius | `blur_expands_input_region_by_declared_radius` |
| 2 | Transform back-maps required input (four-corner oracle) | `affine_input_region_equals_independent_four_corner_oracle` |
| 3 | Infinite generator clamped to finite Final/Stage request | `infinite_generator_clamps_to_requested_final_region` |
| 4 | Unknown plan evaluates same pixels as full domain | `unknown_plan_pixels_match_full_domain_evaluation` |
| 5 | Understated finite rejected by conformance | `understated_finite_declaration_is_rejected_by_conformance` |
| 6 | Unverified plugin output extent is Unknown | `unverified_plugin_output_extent_is_unknown` |
| 7 | Canonical coordinates only; independent of pixel dimensions | `extents_are_canonical_and_independent_of_pixel_dimensions` |
| 8 | No alpha/readback extent derivation | `region_plan_is_independent_of_pixel_content` |
| 9 | Draft/Final share one region function | `draft_and_final_share_one_region_function` |

Additional contract tests (same file, same grain):

| Test | Role |
|---|---|
| `region_plan_denies_unverified_plugin_its_declared_finite_extent` | Trust gate + adoption via `output_extent` only |
| `uv_matrix_is_not_interchangeable_with_canonical_inverse` | Canonical vs UV matrix separation (planner + evaluator) |
| `union_with_unknown_branch_adopts_literal_unknown` | Unknown ∪ Finite → Unknown before hull |
| `unknown_declaration_does_not_shrink_adopted_region_below_requested` | Unknown never crops Final at plan level |
| `blur_after_affine_composes_expanded_then_backmapped_region` | Expand→Affine composition order |
| `out_of_range_finite_source_samples_transparent_black` | OOR transparent black |

## v2 Grok P1 findings → discriminating controls

| P1 # | Finding | Test | Negative / discriminating control |
|---|---|---|---|
| 1 | Affine input region must match independent four-corner back-map | `affine_input_region_equals_independent_four_corner_oracle` | Oracle is inline `f64::min`/`f64::max` on corner products; test calls only `input_regions` |
| 2 | `inverse_uv` ≠ `canonical_inverse` | `uv_matrix_is_not_interchangeable_with_canonical_inverse` | (a) assert whole arrays differ, with index-0/index-4 equality and index-1/index-2/index-3/index-5 inequality; (b) divergent `inverse_uv` → same `RegionPlan`; (c) same `Canvas`; (d) UV coeffs as `canonical_inverse` → `RegionPlan` and `samples` differ |
| 3 | Union with Unknown branch → literal Unknown | `union_with_unknown_branch_adopts_literal_unknown` | Control graph: both branches Conformed/Finite → adopted Finite hull with four bound assertions |
| 4 | Unverified cannot adopt declared Finite | `region_plan_denies_unverified_plugin_its_declared_finite_extent` | Flip only `trust` to Conformed → adopted declared Finite |

## Commands and results

All cargo invocations use `CARGO_TARGET_DIR=/private/tmp/motolii-m4-k0-composer-target` from `/private/tmp/motolii-m4-k0-composer-20260725`.
The final independent Codex integration rerun of clippy and the full workspace used
`CARGO_TARGET_DIR=/private/tmp/motolii-m4-k0-codex-final-target`.

### `cargo fmt --all --check`

Exit code: `0` (clean).

### `cargo clippy --workspace --all-targets --locked -- -D warnings`

Exit code: `0` (clean; no warnings).

### `cargo test -p motolii-render --test k0_region_contract`

```
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### `cargo test -p motolii-render --test k0_region_contract -- --list`

```
affine_input_region_equals_independent_four_corner_oracle: test
blur_after_affine_composes_expanded_then_backmapped_region: test
blur_expands_input_region_by_declared_radius: test
draft_and_final_share_one_region_function: test
extents_are_canonical_and_independent_of_pixel_dimensions: test
infinite_generator_clamps_to_requested_final_region: test
out_of_range_finite_source_samples_transparent_black: test
region_plan_denies_unverified_plugin_its_declared_finite_extent: test
region_plan_is_independent_of_pixel_content: test
understated_finite_declaration_is_rejected_by_conformance: test
union_with_unknown_branch_adopts_literal_unknown: test
unknown_declaration_does_not_shrink_adopted_region_below_requested: test
unknown_plan_pixels_match_full_domain_evaluation: test
unverified_plugin_output_extent_is_unknown: test
uv_matrix_is_not_interchangeable_with_canonical_inverse: test

15 tests, 0 benchmarks
```

### `cargo test --workspace --locked`

Exit code: `0` (full workspace green, including K0 `15 passed; 0 failed`).

### `./scripts/check-docs.sh`

```
OK: docs整合チェック全項目通過
```

### `git status --porcelain`

```
?? crates/motolii-render/tests/k0_region_contract.rs
?? docs/spikes/m4-k0-region-contract.md
```

### `git diff --stat eb3ed25572603fb7b2a982fd99c08bc6ffce92e1`

No output (no tracked file modified).

### `git rev-parse HEAD && git rev-parse refs/heads/codex/m4-k0-composer-fallback-20260725`

```
eb3ed25572603fb7b2a982fd99c08bc6ffce92e1
eb3ed25572603fb7b2a982fd99c08bc6ffce92e1
```

## Gate checks

Executed directly after repair:

- `grep -n "allow(\|expect(\|unsafe\|#\[ignore\]\|todo!\|unimplemented!" crates/motolii-render/tests/k0_region_contract.rs`
  - `.expect` hits are existing Option/Result unwraps; no `allow`/`unsafe`/`todo!`/`unimplemented!`/`#[ignore]` patterns.
- `grep -c "^#\[test\]" crates/motolii-render/tests/k0_region_contract.rs`
  - `15`
- `grep -n "invert_affine" crates/motolii-render/tests/k0_region_contract.rs`
  - only helper definition and `affine_forward_extent` usage.
- `grep -n "let _ =" crates/motolii-render/tests/k0_region_contract.rs`
  - no output
- `grep -n "DRAFT" crates/motolii-render/tests/k0_region_contract.rs`
  - only in `draft_and_final_share_one_region_function`.

## Grok REJECT (P0=0 P1=5 P2=4) → repair

| Scope | Test / item | Repair details |
|---|---|---|
| P1 | `out_of_range_finite_source_samples_transparent_black` | Replaced the inverse-of-forward fixture with the literal magnifying `canonical_inverse = [2, 0, 0, 0, 2, 0]`, classified mapped output-canonical positions in source UV space, and added exact out-of-range/in-range counts (`12`/`4`) plus per-pixel black/non-black assertions. |
| P1 | `uv_matrix_is_not_interchangeable_with_canonical_inverse` | Fed `canonical_inverse` directly into the `affine.rs` conjugation formula without an extra inversion; asserted whole-array inequality, index-0/index-4 equality, and index-1/index-2/index-3/index-5 inequality; then exercised the matching-inverse wrong declaration against one fully nonzero shared source. |
| P1 | `understated_finite_declaration_is_rejected_by_conformance` | Added property-based check that the differing sample index maps to a canonically excluded texel, not a fixed numeric index. |
| P1 | `extents_are_canonical_and_independent_of_pixel_dimensions` | Reused a single `region_plan` across 8×8 and 32×32 canvases whose source pixels are keyed to the same `TextureId(0)`; asserted exact in-region counts (`16/64`, `256/1024`) and exact fractional consistency `count8 * 32 * 32 == count32 * 8 * 8` plus canonical in/out pixel correctness per canvas. |
| P1 | `region_plan_is_independent_of_pixel_content` | Removed fixture discard path and keyed both fixtures to `out`; required fixture difference and asserted evaluator output differences under divergent sources with identical plan and dimensions. |
| P2 | report refresh | Rebuilt `docs/spikes/m4-k0-region-contract.md` from this rerun (`git rev-parse` branch/SHA, commands, test list, and freeze gates). |
| P2 | test-11 matching-inverse control | Documented and implemented control using matching `inverse_uv` path; this control is now non-vacuous after `canonical_inverse` correction. |
| P2 | Accepted-conformance | Explicit non-goal for this grain; deferred. |
| P2 | singular-affine Unknown + promotion-API-dependent post-reject trust assertions | Explicit non-goal for this grain; deferred. |

## Final independent review

`cursor-grok-4.5-high` re-audited the approved v4 order and final two-file artifact read-only.
It confirmed all five prior P1 findings closed and returned:

```text
VERDICT: ACCEPT
P0=0 / P1=0 / P2=1
```

The sole P2 was this report's inaccurate wording that the 8×8 and 32×32 canvases were keyed
to the same output texture. The row above now states the implemented source key,
`TextureId(0)`. No code change was required after acceptance.

## What this spike does **not** prove

- No shipped runtime API, public types, or `motolii-testkit` promotion
- No real ROI optimization, cache keys, `ResourceLedger`, budget, or admission (K1a/K1b)
- No plugin / Document / schema / serde / persistence contract
- No GPU execution path in this spike
- No K1+ implementation work

## FREEZE DECISION: PASS

All integration gates in this report are green: 15/15 test count, list count, 2-file scope, fmt,
workspace clippy with warnings denied, full workspace tests, docs check, grep/static gates, and
branch/SHA checks.
