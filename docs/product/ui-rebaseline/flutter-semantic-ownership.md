# Semantic ownership: one owner per concept (2026-09-27, working checklist)

Purpose: not to thin out Flutter, but so that Flutter, Motolii Live (JS), the Script API, Vism and any later frontend share the same Host semantics.
Flutter owns how Motolii feels: gesture, hover, focus, layout, dock, visual state, Instrument bodies, temporary preview presentation.
The Host owns what Motolii means: what a kind is, who an edit reaches, what a selection of keys means, what is dirty, what identity persists.

Classes: A Host or Document semantic, B Session or operation semantic, C workspace or UI policy, D presentation or interaction, E Camera related and deferred (untouched).

## Moved to the host in this pass

| Concept | Class | Before | Now | Proof |
|---|---|---|---|---|
| Sequence delay spread | A | Dart turned the curve into a delay per layer | `sequence` and `previewSequence` take layers and a curve; `editor::sequence_delays` | Rust `a_sequence_given_only_a_curve_spreads_the_delay_itself` |
| Edits to a selection | B | three Dart copies (Classic Inspector, Transform Instrument, effect sheet) each computed one value per layer | `previewProperties` takes one edit with a spread: `offset` for a drag, `typed` for only the changed axes, `absolute` for every target; locked layers refused, committed values are the base | Rust `a_selection_is_edited_by_the_host_in_the_way_the_edit_says` |
| Blend targets | A | Dart chose selected, unlocked, non-camera layers and sent per layer edits | status `blendTargets`; `applyBlend` applies to those that differ as one step; `previewBlend` without a layer previews on the last target | Rust `a_blend_reaches_the_selected_layers_that_can_take_it` |
| Key intervals of a selection | A | Dart walked layers and rows to find the intervals | status `easeIntervals` | Rust `selected_keys_mean_intervals_the_host_names` |
| What can be created | A | a Dart list of kinds, names, details and shelves, beside a Rust match | one table `editor::create::kinds`; `create` reads it, status `createKinds` publishes it, the Browser projects it and adds only the mark | Rust `every_listed_kind_can_be_created_and_is_published` |
| Dirty | A | already the host's: status `dirty` from the saved signature. Dart only shows it | unchanged, confirmed | `snapshot_cache` tests |

Two ids are still supplied by the caller, by design: page and block ids of Notes are minted by the frontend and validated by the host (a duplicate is refused, a repeat replaces). That is a client-supplied identity contract, the same for any frontend, not a hidden rule.

## Stays in Flutter, on purpose

| What | Class |
|---|---|
| which Desk follows which selection (keys to Ease, several layers to Ease as a Sequence, a blend change to Blend); the drawer, the default, the panel placement | C |
| dock, window, focus, scale, saved layout | C |
| the user's own Browser state: favorites as collection 1, collections 2 to 7, the digit keys, recent, saved searches, tags and ranges, kept in the desk settings | C |
| curve geometry, handle drag, ghost marks on a curve, peek and audition, preview coalescing | D |
| how a drag on a Transform ring or a Layout handle becomes a value (the gesture is Flutter; who it reaches is the host) | D |
| Notes card size and clamps, the dotted ground, zoom | D |

## Semantic debt still in Dart (small, listed so a second frontend knows)

| What | Note |
|---|---|
| History position: the current point is the last entry whose head is at or before the head | could be a status field `historyAt` |
| Notes reference block: its label is composed in Dart from a layer name and a range; the carry-over of earlier notes is a Dart migration | the label could be read from the layer; the migration goes with Classic |
| Scale link: keeping the ratio while one axis changes is computed in Dart | a host rule when the Instrument and Script both need it |
| Media kind of an asset (video, image, audio) is derived from the mime in Dart | the host could say `kind` |
| Files shelf lists folders with the Dart file system | an alternative frontend must list its own; a host service if Live needs it |
| Dart keeps a table of the host's operation names (`bridge/protocol.dart`), so a new host operation needs one line there | mirror of the host, not a rule |
| Depth: camera orbit math | E, untouched |

## Motolii Live compatibility note

Reusable by Live today, through the same `op` path Flutter uses (the JS prelude already calls `create`, `setProperty`, `setAttrs`, `select`, `applyPalette`):
- `create` with any id in the status `createKinds` table (the JS names line up with those ids; `bezier` and `stage` are in the table and not yet named in the prelude)
- `previewProperties` with a `spread`, then `commitPreview` or `cancelPreview`: one edit that reaches the selection the same way for Flutter and a script
- `applyBlend` and `previewBlend` for the blend targets
- `sequence` with a curve
- `dirty`, `blendTargets` and `easeIntervals` in the status

Still Flutter-only interpretation: which Desk follows which selection, the Browser user state, gesture to value, Notes composition, History position.

Minimal Live binding to expose these later: a `selection` object with `edit(property, value, {spread})`, `blend(mode)` and `easeIntervals()`; a `kinds()` list from `createKinds`; `dirty` as a property. Nothing more is needed to avoid copying Flutter's rules.

Vism and Cassette: the kind table is now the one place a shelf of new things would register, so a Vism that adds a thing adds a row, not a Flutter switch. No new runtime was made.
