# Motolii: ordinary, comfortable editing

## Boundary

User request, 2026-09-05: improve implementation, usability, maintainability and iteration speed together. Keep `motolii/AGENTS.md` unchanged. Preserve vgpu-derived Vism and the single re_renderer 3D Stage; all artwork belongs to that world. Document/Intent remains the editing authority.

User intent is the governing product criterion: retain AE's successful capabilities without requiring its workarounds. Prefer direct editing, discoverable next actions and reversible experimentation over reproducing legacy steps. The constitution serves that intent.

User review: the independent HTML concept was not good enough and is not adopted. Upgrade existing product assets incrementally, retaining their established layout and interaction conventions. Do not pursue a replacement UI based on that mock.

User priority correction: keyframes, UI label overlap (not artwork text), persistent color wheel, and Photoshop/CLIP STUDIO-style layer masking take precedence over further internal cleanup. Prior AI-authored constraints are not user decisions. Preserve the explicit constitution/core, but do not defend broken or missing interactions as intentional design.

## Current user-facing failures

| Area | Evidence | Required next proof |
|---|---|---|
| UI labels | Real window: Colors hex labels paint above their palette, over the hue wheel. Disabling card animation and moving labels to normal flow did not remove duplicates; both probes reverted. | Trace layout/paint ownership in Blitz and reproduce with the actual control family; verify uncluttered labels after selection/resize/scroll |
| Color wheel | browser.rs replaces ColorWheel with a message when wheel_slot is None; real window confirms it disappears with no layer selected. | Wheel remains available in Colors without target; choosing/previewing a color must not mutate an unrelated layer |
| Layer masks | Document has distinct AddMask/masks data, but Create adds a preset rectangle. No mask selection/editing path was found in Inspector, Stage, Timeline shell or Session. | Add/select/edit/disable/remove a layer mask through its layer, with direct visible feedback and Undo |
| Spatial masks | engine/render.rs::apply_masks_to_layer returns unchanged non-texture content with a flatten-required diagnostic. | Preserve the user's native 3D requirement; do not call planar flattening full support |
| Keyframes | User reports malfunction; current unit tests do not identify the concrete live failure. | Observe create/select/move/edit/delete/Undo at current playhead; compare Stage and stored values |

Mask reference: [Photoshop layer masks](https://helpx.adobe.com/photoshop/desktop/create-masks/layer-masks/add-layer-masks.html) and [CLIP STUDIO layer masks](https://help.clip-studio.com/en-us/manual_en/180_layers/Layer_masks.htm). Common interaction: masks belong to layers, have a selectable thumbnail, and can be edited non-destructively. Do not silently equate Photoshop grayscale editing with CLIP STUDIO's alpha-based editing.

## Execution

1. Verify the current development process and existing regression suite. Reuse caches and one feature/profile; do not use repeated full builds as progress checks. A running app is not evidence of a running reload server.
2. Reproduce failures against current code. `live-operation-findings.tsv` is an investigation index, not current proof: several listed gaps already have implementations. Fix shared editing/interaction owners and retain regressions for observed failures.
3. Exercise creation, selection, value editing, drag cancellation, duplication, Undo/Redo and save/open as connected operations. Check cross-panel and interrupted operations as well as happy paths.
4. Preserve stable control locations, including Transform. User correction: intent priority does not mean moving controls according to the selected content. The attempted Inspector reorder was rejected and its source change fully reverted. Reconsider architecture before further UI rearrangement; inspect actual windows and leave visual adoption to the user.
5. Record passed checks, remaining failures and visual evidence separately. No claim of whole-product completion from compilation or a partial test pass.

## Existing rulers

- `motolii/reference/ordinary-upstream.tsv`, `ux-chaos.tsv`, `function-contracts.json`: existing interaction contracts and upstream mappings.
- `docs/ideal.md`: intent, progressive disclosure, immediate feedback and separation of observation from artwork.
- `motolii/reference/vgpu-vism.md`: Vism declarations and reload boundaries.
- `scripts/reload-runtime.py`, `reload-step.py`, `reload-runtime-test.py`: pinned runtime, reuse and exception receipts.

## Initial evidence

- Working tree clean at start; HEAD `429c7a5a`.
- Runtime doctor verifies the pinned executable but reports no warm server. Existing app PID 2203 is open; its document must not be discarded to establish a new baseline.
- Actual Inspector requires scrolling to reach selected text content and size.
- First validation: existing UI binary tests, default features and dev/test profile, existing target directory. Purpose: identify current behavioral failures across shared editing surfaces; hotpatch cannot run the headless regression harness. This is one baseline, not a per-edit full-build loop.
- Baseline completed: `cargo test --locked -p motolii --bin motolii`, 213 passed, zero failures; compilation 54.44 s, execution 51.32 s. Historical test PID 83265 is still blocked in an older binary; it is not evidence that current code fails. No other task's process was stopped.
- Runtime launcher now detects the exact open development app separately from its server; an orphan app cannot silently cause another baseline and window. Python runtime regressions: 9 passed. Live doctor reports PID 2203 as `unmanaged_app`.
- Keep disk/cache accounting in `motolii-dx.sh doctor`, not every `serve`/reuse request. Runtime guards still verify executable checksum, stopped compilers, server identity/count, lock ownership and open orphan apps. Reuse must not run a build, mutate a permit or scan the entire target/source cache.

## Architecture investigation: publication after editing

Observed in current code, not inferred from file length:

| Responsibility | Existing owner/evidence | Decision |
|---|---|---|
| Atomic artwork edits and history | Document::apply_all, used by commands and Session::apply_blocks | Retain; do not reproduce in a new UI model |
| Property write policy and operation composition | functions/lens.rs, compose.rs, property_edit.rs | Retain the shared policy; inspect coverage rather than replace the abstractions wholesale |
| Layer-list / flags / Timeline projection publication | app::refresh_layer_projection already performs the complete publication, but browser::spawn_layer and timeline_shell flag/fold handlers repeat it | Route those copies through the existing owner; no new cache or event bus |
| Selection and per-window projections | Session::selection plus Panes::selected and multiple Timeline message paths | Further investigation required: mirrors may be justified adapters, but synchronization obligations are scattered |
| Retained GPU surfaces and hot entry points | MountStore, SurfaceState, HotFn in mount.rs | Retain; wholesale UI replacement would discard working state-preservation infrastructure |

First structural cut: remove duplicated publication from creation, flag toggles and row expansion. Preserve the same projection functions, order and revision notification. Existing creation/Undo/Timeline GUI tests are the ruler. This does not establish an event-driven single publication boundary for the whole product; that remains open.

## Verified incremental changes

- Creation, flag toggles and expansion now use `app::refresh_layer_projection`; 16 net UI lines removed, no layout/interaction redesign.
- Targeted GUI regression run: `cargo test --locked -p motolii --bin motolii ui::gui:: -- --skip the_window_still_takes_orders_after_any_storm`: 67 passed, zero failures, 4.23 s execution. The random storm was already included in the initial 213-test baseline; it was not rerun for this publication-only refactor.
- Last combined Rust patch: CLI reports 3741 ms, app accepted epoch 3 with PID 4419, Document owner 4949356560 and history [12,0]; GPU device 6961116553929376407 retained. This proves patch acceptance/continuity, not visual or gesture acceptance of every changed route.
- Runtime regression suite: 11 passed. `scripts/motolii-dx.sh serve` reused live server PID 3506 in 305 ms, exit 0, without a new baseline.
- Constitution unchanged. HTML concept rejected; no product stylesheet or Inspector layout changes remain.

## Editing failure feedback

`session::noted` previously printed failures only to PROBE and did not notify the view. Its eight call sites (numeric commit, projection, freeze, palette, asset removal and composition controls) now pass the existing project notice explicitly. Failure publishes the StoreError and refreshes status; no new dialog or hidden context dependency. Existing partial-rejection notices on successful operations remain intact.

Regression: a real locked-layer removal rejection leaves Document revision unchanged, preserves its error reason in the notice and increments the view notification. `rejected_edit_is_published_without_changing_document_history` passed (one test, 0.01 s). App accepted epochs 5/6 with PID 4419, Document owner 4949356560, history [12,0] and the same GPU device. Individual native failure presentation is not yet visually accepted. Other log-only errors outside `noted` remain to be addressed.

## UI text overlap: reproduced dependency defect

Pinned Blitz `64eb278`, `blitz-dom/src/layout/damage.rs::flush_styles_to_layout_impl`: when a node ceases being a stacking-context root, descendants are appended to the parent context, but the node's previous `stacking_context` is not cleared. `blitz-paint/src/render.rs::draw_children` draws both lists. This explains why label backgrounds and text appear again above their actual cards.

`ui::gui::palette_labels_have_one_paint_owner_after_animation` walks the same paint graph from the root element. Opening Colors alone passed. Changing its grid opacity 0.5 -> 1 reproduced **three paint owners for one label**, expected one. This initially failed and now passes with the vendored repair. CSS animation/position workarounds were reverted.

Candidate upstream repair is the single line in `reference/blitz-stale-stacking-context.patch`. It is applied through the pinned vendored crate and verified below. Preserve Dioxus/Blitz for LLM authoring and Rust/wgpu/re_renderer GPU sharing; no framework migration or CPU image replacement.

Integration update: `vendor/blitz-dom` contains the pinned crate and its licenses; Cargo workspace inheritance is expanded, path-only upstream dependencies retain the same pinned Git source. Only `src/layout/damage.rs` differs among upstream source files (one added line), checked byte-for-byte. Cargo.lock changes only this crate's source. The three-paint-owner regression now passes (1.71 s); initial dependency test compilation took 43.18 s.

Native restart need: Cargo dependency graph changed from the Git crate to the patched local crate. The guarded CLI explicitly refused `CONFIGURATION / configuration_graph_refresh_required`; it cannot reuse the old dependency graph as a normal thin patch. Save the current project, establish one new baseline with the same features/profile/cache, reopen the project, then return to warm patching. Document history and ephemeral selection cannot survive that restart. Full UI tests passed: 216/216, including the random storm, 64.17 s. Native PID 11800 / server 10781 reopened the saved project. Selecting Text while Colors is visible shows the hue wheel and palette with no duplicated Hex labels above it. This verifies the reported duplicate-label route, not all possible layout defects.

Latest user direction: Ableton-like consistent skin, Figma-like direct interaction, AE-level compositing capability. These are design aims, not licenses to preserve provisional 3D gizmos or invent constraints. Upgrade the existing assets and evaluate concrete operations.

Critical correction: the group-layer example primarily calls for features that **solve problems structurally**, eliminating the underlying need for workarounds or restrictions. It is not primarily an instruction to bundle several operations under one meaning. Evaluate representation, ownership, relationships and evaluation paths that make the desired capability natural; do not reduce this direction to shortcut collections or UI consolidation.

User explicitly authorizes substantial code restructuring: code is not the thing to preserve. Preserve cohesive intent, Ableton-like organized density, Figma-like short direct editing paths, and useful AE conventions such as layers and keyframes. AE extensions are evidence of recurring authoring needs and host friction; standardize common operations rather than requiring add-ons to make ordinary work convenient. Earlier mock rejection is not a veto on product improvements or structural rewrites. The explicit Dioxus/Blitz, Rust/wgpu, re_renderer Stage and Vism foundations remain.

## Parallel persona batch

User authorized parallel personas and broad code-first inspection. Three read-only audits covered motion/keyframes, painting/masks/color, and direct editing/focus. Comments were clues to assumptions, not an authority or an automatic deletion list. Implementation ownership was disjoint: easing, palette, shared Field; root owned grouped key movement and combined validation.

Implemented and individually passing within the current combined run:
- Multi-key nudges reuse the grouped drag preparation, write each track once, and synchronize selected-key identities. Regression includes overlapping aggregate/property selection and one Undo/Redo.
- Multiple easing intervals are grouped by layer/property before applying one replacement. Regression preserves unselected metadata and verifies rejection/Redo preservation.
- Palette commits honor the explicitly focused ColorSlot, preserving gradient structure and its other endpoint. Regression invokes the actual commit route and Undo.
- Tab/Shift+Tab commits an open field and permits the existing native focus traversal. GUI regression edits X -> Y -> X and verifies final values; multiline Enter contract retained.
- Moving a key onto an occupied time preserves the moved key in both directions, using the track's existing replacement semantics. Numeric and content moves insert stationary keys before moved keys. Numeric collision regression verifies both directions and Undo.

Still open from the same audits: F9 incoming/outgoing selection semantics; invalid input draft retention; selection-wide transform pivot; picking through locked objects; persistent unbound color picker; editable mask/path lifecycle and native spatial coverage. These are not marked complete by the fixes above. Native operation verification of this batch remains separate from test results.

Second batch verified: 228 tests passed, zero failures, 30.02 s. F9 now targets selected incoming/outgoing endpoints, retaining the opposite existing Bezier handle and converting other interpolation without a new refusal. Invalid Number/Hex drafts remain editable on Enter/Tab/Shift+Tab; correction and Escape are covered. Outside-click selection changes can still discard an invalid draft and remain open work. ColorWheel accepts an optional target: unbound exploration commits locally without Document mutation and cancels transient values safely. Native selection clear retained the wheel; choosing hue/SV changed the displayed trial color while artwork stayed unchanged. Trial color is component-local, not a saved brush preference.

Latest UI-text instruction: avoid explanatory prose in the product. Colors now uses short target-state labels and no duplicate tutorial/status sentences; verified in the live window. `.rcount` uses a minimum height instead of forcing multiline status into a single-row height.
